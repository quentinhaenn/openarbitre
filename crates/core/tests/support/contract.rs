//! Contract suite of the `Store` port and its reference model, generic over the adapter
//! (PROP-ST02-9; plan T-6). Included by `tests/store.rs` and `crates/sqlite/tests/contract.rs`.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use arbitre_core::*;
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestRunner};

use crate::support::config;

/// An adapter under test; owns its handle (and, on SQLite, its path).
pub trait Harness {
    type Store: Store;
    fn fresh() -> Self;
    fn store(&self) -> &Self::Store;
    /// Drops the handle and opens the same store again; panics on failure.
    fn reopen(&mut self);
}

/// Polls `f` once with a no-op waker; a store future must be ready (RG-ST02-6.3).
pub fn ready<F: Future>(f: F) -> F::Output {
    match pin!(f).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("store future not ready on first poll (RG-ST02-6.3)"),
    }
}

/// Runs the suite on `H`: the model-based property (1,000 sequences, `support::config()`),
/// then the named cases.
pub fn run<H: Harness>() {
    let cfg = ProptestConfig {
        failure_persistence: None,
        ..config()
    };
    TestRunner::new(cfg)
        .run(
            &(pools(), prop::collection::vec(op(), 0..=50)),
            |(p, ops)| {
                let mut r = Run::<H>::new();
                r.check(&p);
                for o in &ops {
                    r.step(&p, o);
                    r.check(&p);
                }
                Ok(())
            },
        )
        .unwrap();
    named::<H>();
}

// ---------------------------------------------------------------- generators

/// Names, values and domains of one sequence (INV-ST02-6: random content, few distinct items).
#[derive(Debug, Clone)]
struct Pools {
    modules: Vec<String>,
    cats: Vec<String>,
    ids: Vec<String>,
    fields: Vec<String>,
    values: Vec<Vec<u8>>,
}

impl Pools {
    fn rec(&self, n: u8) -> RecordId {
        let n = n as usize % 12;
        RecordId {
            module: self.modules[n % 2].clone(),
            category: self.cats[n / 2 % 2].clone(),
            id: self.ids[n / 4].clone(),
        }
    }
    fn field(&self, n: usize) -> Field {
        Field(self.fields[n % 3].clone())
    }
    fn value(&self, n: u8) -> Value {
        Value(self.values[n as usize % 3].clone())
    }
}

fn pools() -> impl Strategy<Value = Pools> {
    let name = "[a-c\\x00é😀]{1,3}";
    let value = prop::collection::vec(prop_oneof![Just(0u8), Just(0xFF), any::<u8>()], 0..3);
    (
        prop::collection::vec(name, 2),
        prop::collection::vec(name, 2),
        prop::collection::vec(name, 3),
        prop::collection::vec(name, 3),
        prop::collection::vec(value, 3),
    )
        .prop_map(|(modules, cats, ids, fields, values)| Pools {
            modules,
            cats,
            ids,
            fields,
            values,
        })
}

#[derive(Debug, Clone)]
enum Op {
    /// `pt`, then `(record, kind, value, author)` per change.
    Local(u64, Vec<(u8, u8, u8, u8)>),
    Recv(Vec<Item>),
    Mark(u8, (u64, u32)),
    Read(u8),
    Reopen,
}

#[derive(Debug, Clone)]
enum Item {
    /// Peer write (peer index 0 = the local device itself), `seen` entries `(key, l, c)`.
    Fresh {
        rec: u8,
        kind: u8,
        val: u8,
        author: u8,
        stamp: (u64, u32, u8),
        seen: Vec<(u8, u64, u32)>,
    },
    /// Copy of a stored write (local ones included).
    Dup(u8),
    /// Stored write with another content: peer anomaly, or Q-A violation if it is local.
    Alt(u8),
    /// Write with an empty identifier.
    Bad,
}

fn op() -> impl Strategy<Value = Op> {
    let seen = prop::collection::vec((0u8..4, 0u64..12, 0u32..3), 0..4);
    let fresh = (
        any::<u8>(),
        0u8..4,
        0u8..3,
        0u8..4,
        (0u64..12, 0u32..3, 0u8..6),
        seen,
    )
        .prop_map(|(rec, kind, val, author, stamp, seen)| Item::Fresh {
            rec,
            kind,
            val,
            author,
            stamp,
            seen,
        });
    let item = prop_oneof![
        6 => fresh,
        2 => any::<u8>().prop_map(Item::Dup),
        2 => any::<u8>().prop_map(Item::Alt),
        1 => Just(Item::Bad),
    ];
    let change = (any::<u8>(), 0u8..30, 0u8..3, 0u8..4);
    prop_oneof![
        5 => (0u64..8, prop::collection::vec(change, 1..=5)).prop_map(|(pt, c)| Op::Local(pt, c)),
        5 => prop::collection::vec(item, 0..=6).prop_map(Op::Recv),
        2 => (0u8..3, (0u64..4, 0u32..3)).prop_map(|(p, w)| Op::Mark(p, w)),
        2 => any::<u8>().prop_map(Op::Read),
        1 => Just(Op::Reopen),
    ]
}

fn dev(n: u128) -> DeviceId {
    DeviceId::from_u128(n)
}

/// Peers 1 (< `D`), `u128::MAX` (> `D`) and 2; index 0 is the local device.
fn peer(d: DeviceId, i: u8) -> DeviceId {
    match i {
        0 => d,
        1 | 2 => dev(1),
        3 | 4 => dev(u128::MAX),
        _ => dev(2),
    }
}

const PEERS: [u128; 3] = [1, u128::MAX, 2];

const AUTHORS: [Author; 4] = [
    Author::CliUser,
    Author::UiUser,
    Author::Assistant,
    Author::System,
];

fn local_changes(p: &Pools, cs: &[(u8, u8, u8, u8)]) -> Vec<LocalChange> {
    cs.iter()
        .map(|&(rec, kind, val, author)| LocalChange {
            record: p.rec(rec),
            change: match kind {
                0..24 => Change::Field {
                    name: p.field(kind as usize),
                    value: p.value(val),
                },
                24..29 => Change::Delete,
                _ => Change::Field {
                    name: Field(String::new()),
                    value: Value(vec![]),
                },
            },
            author: AUTHORS[author as usize % 4],
        })
        .collect()
}

/// Received batch; `seen` keys from {`Delete`, own field, other field, `Field("")`} (DA F-1).
fn batch(p: &Pools, d: DeviceId, stored: &[Write], items: &[Item]) -> Vec<Write> {
    let mut out = Vec::new();
    for it in items {
        match it {
            Item::Fresh {
                rec,
                kind,
                val,
                author,
                stamp: (l, c, a),
                seen,
            } => {
                let (change, own) = match kind {
                    0..3 => (
                        Change::Field {
                            name: p.field(*kind as usize),
                            value: p.value(*val),
                        },
                        *kind as usize,
                    ),
                    _ => (Change::Delete, 0),
                };
                let stamp = Stamp {
                    l: *l,
                    c: *c,
                    a: peer(d, *a),
                };
                let seen = seen
                    .iter()
                    .map(|&(key, sl, sc)| {
                        let key = match key {
                            0 => TargetKind::Delete,
                            1 => TargetKind::Field(p.field(own)),
                            2 => TargetKind::Field(p.field(own + 1)),
                            _ => TargetKind::Field(Field(String::new())),
                        };
                        let m = Stamp {
                            l: sl,
                            c: sc,
                            a: dev(1),
                        };
                        (key, m)
                    })
                    .filter(|(_, m)| *m < stamp)
                    .collect();
                out.push(Write {
                    record: p.rec(*rec),
                    change,
                    stamp,
                    seen,
                    author: AUTHORS[*author as usize % 4],
                });
            }
            Item::Dup(i) if !stored.is_empty() => {
                out.push(stored[*i as usize % stored.len()].clone());
            }
            Item::Alt(i) if !stored.is_empty() => {
                let mut w = stored[*i as usize % stored.len()].clone();
                match &mut w.change {
                    Change::Field { value, .. } => value.0.push(1),
                    Change::Delete => w.author = AUTHORS[(w.author as usize + 1) % 4],
                }
                out.push(w);
            }
            Item::Bad => out.push(Write {
                record: RecordId {
                    module: String::new(),
                    category: "g".into(),
                    id: "i".into(),
                },
                change: Change::Delete,
                stamp: Stamp {
                    l: 1,
                    c: 0,
                    a: dev(1),
                },
                seen: Seen::new(),
                author: Author::CliUser,
            }),
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------- reference model

/// ST-01 `Log` of `D`, stored writes in commit order, watermarks.
struct Model {
    d: DeviceId,
    log: Log,
    stored: Vec<Write>,
    pos: Vec<Position>,
    marks: BTreeMap<DeviceId, Watermark>,
}

impl Model {
    /// All or none: works on a clone of the log.
    fn local(&mut self, pt: u64, cs: &[LocalChange]) -> Result<Vec<Write>, Rejection> {
        let mut log = self.log.clone();
        let mut out = Vec::new();
        for (index, c) in cs.iter().enumerate() {
            let w = log
                .write(pt, c.record.clone(), c.change.clone(), c.author)
                .map_err(|error| Rejection::Local { index, error })?;
            out.push(w);
        }
        self.log = log;
        self.stored.extend(out.iter().cloned());
        Ok(out)
    }

    /// Q-A guard on a clone, dropped on rejection (DA F-3).
    fn recv(&mut self, b: &[Write]) -> Result<(Report, Vec<Write>), Rejection> {
        let mut log = self.log.clone();
        let report = log.merge(b).map_err(Rejection::Batch)?;
        let reused: BTreeSet<(Target, Stamp)> = report
            .new_anomalies
            .iter()
            .filter(|a| a.stamp.a == self.d)
            .map(|a| (a.target.clone(), a.stamp))
            .collect();
        if let Some(index) = b
            .iter()
            .position(|w| reused.contains(&(w.target(), w.stamp)))
        {
            return Err(Rejection::LocalIdentityReused { index });
        }
        let mut new: Vec<Write> = Vec::new();
        for w in b {
            if !self.stored.contains(w) && !new.contains(w) {
                new.push(w.clone());
            }
        }
        self.log = log;
        self.stored.extend(new.iter().cloned());
        Ok((report, new))
    }
}

struct Run<H: Harness> {
    h: H,
    m: Model,
}

impl<H: Harness> Run<H> {
    fn new() -> Self {
        let h = H::fresh();
        let d = ready(h.store().device()).unwrap();
        Self {
            h,
            m: Model {
                d,
                log: Log::new(d),
                stored: Vec::new(),
                pos: Vec::new(),
                marks: BTreeMap::new(),
            },
        }
    }

    fn last(&self) -> Position {
        self.m.pos.last().copied().unwrap_or(Position::START)
    }

    /// The next `writes_since(last)` is exactly `new`, after `last` (DA F-8).
    fn expect_tail(&self, last: Position, new: &[Write]) {
        let tail = ready(self.h.store().writes_since(last)).unwrap();
        assert!(tail.iter().all(|(p, _)| *p > last));
        assert_eq!(tail.into_iter().map(|(_, w)| w).collect::<Vec<_>>(), new);
    }

    fn step(&mut self, p: &Pools, op: &Op) {
        let last = self.last();
        match op {
            Op::Local(pt, cs) => {
                let cs = local_changes(p, cs);
                let got = ready(self.h.store().apply_local(*pt, &cs));
                match self.m.local(*pt, &cs) {
                    Ok(ws) => {
                        assert_eq!(got, Ok(ws.clone()));
                        self.expect_tail(last, &ws);
                    }
                    Err(r) => assert_eq!(got, Err(ApplyError::Rejected(r))),
                }
            }
            Op::Recv(items) => {
                let b = batch(p, self.m.d, &self.m.stored, items);
                let got = ready(self.h.store().apply(&b));
                match self.m.recv(&b) {
                    Ok((report, new)) => {
                        assert_eq!(got, Ok(report));
                        self.expect_tail(last, &new);
                    }
                    Err(r) => assert_eq!(got, Err(ApplyError::Rejected(r))),
                }
            }
            Op::Mark(i, (l, c)) => {
                let (id, to) = (dev(PEERS[*i as usize % 3]), Watermark(*l, *c));
                ready(self.h.store().advance_watermark(id, to)).unwrap();
                let e = self.m.marks.entry(id).or_insert(to);
                *e = (*e).max(to);
            }
            Op::Read(i) => self.since(*i as usize % (self.m.pos.len() + 1)),
            Op::Reopen => self.h.reopen(),
        }
    }

    /// `writes_since` from the position of the `i`-th stored write (0 = start).
    fn since(&self, i: usize) {
        let from = if i == 0 {
            Position::START
        } else {
            self.m.pos[i - 1]
        };
        let got = ready(self.h.store().writes_since(from)).unwrap();
        let want: Vec<_> = self.m.pos[i..]
            .iter()
            .copied()
            .zip(self.m.stored[i..].iter().cloned())
            .collect();
        assert_eq!(got, want);
    }

    /// Every read against the reference; positions by order, stable across steps (RG-ST02-5.3).
    fn check(&mut self, p: &Pools) {
        let s = self.h.store();
        assert_eq!(ready(s.device()), Ok(self.m.d));
        let all = ready(s.writes_since(Position::START)).unwrap();
        let pos: Vec<Position> = all.iter().map(|(p, _)| *p).collect();
        assert!(pos.first().is_none_or(|p| *p > Position::START));
        assert!(
            pos.windows(2).all(|w| w[0] < w[1]),
            "positions not increasing"
        );
        let kept = self.m.pos.len().min(pos.len());
        assert_eq!(pos[..kept], self.m.pos[..], "positions moved");
        let writes: Vec<Write> = all.into_iter().map(|(_, w)| w).collect();
        assert_eq!(writes, self.m.stored);
        self.m.pos = pos;
        let n = self.m.pos.len();
        for i in [0, n / 2, n] {
            self.since(i);
        }
        let s = self.h.store();
        assert_eq!(ready(s.writes_since(Position(u64::MAX))), Ok(vec![]));

        let log_writes = self.m.log.writes();
        let proj = self.m.log.projection();
        let mut recs: Vec<RecordId> = (0..12).map(|n| p.rec(n)).collect();
        recs.push(rid_in("?", "?", "?"));
        for r in &recs {
            let want: Vec<Write> = log_writes
                .iter()
                .filter(|w| w.record == *r)
                .cloned()
                .collect();
            assert_eq!(ready(s.writes_of(r)), Ok(want), "writes_of {r:?}");
            assert_eq!(ready(s.state(r)), Ok(proj.get(r).cloned()), "state {r:?}");
        }
        let cats: BTreeSet<_> = recs.iter().map(|r| (&r.module, &r.category)).collect();
        for (m, c) in cats {
            let want: Vec<_> = proj
                .iter()
                .filter(|(r, _)| r.module == *m && r.category == *c)
                .map(|(r, st)| (r.clone(), st.clone()))
                .collect();
            assert_eq!(ready(s.records(m, c)), Ok(want), "records {m:?} {c:?}");
        }
        for n in PEERS.into_iter().chain([99]) {
            let id = dev(n);
            let want = self.m.marks.get(&id).copied();
            assert_eq!(ready(s.watermark(id)), Ok(want), "watermark {n}");
        }
    }
}

// ---------------------------------------------------------------- named cases

fn rid_in(module: &str, category: &str, id: &str) -> RecordId {
    RecordId {
        module: module.into(),
        category: category.into(),
        id: id.into(),
    }
}

fn rid(id: &str) -> RecordId {
    rid_in("m", "g", id)
}

fn put(id: &str, name: &str, value: &[u8]) -> LocalChange {
    LocalChange {
        record: rid(id),
        change: Change::Field {
            name: Field(name.into()),
            value: Value(value.to_vec()),
        },
        author: Author::CliUser,
    }
}

fn del(id: &str) -> LocalChange {
    LocalChange {
        record: rid(id),
        change: Change::Delete,
        author: Author::CliUser,
    }
}

fn stamp(l: u64, c: u32, a: DeviceId) -> Stamp {
    Stamp { l, c, a }
}

/// Received field write with an empty `seen`.
fn wr(id: &str, name: &str, value: &[u8], st: Stamp) -> Write {
    let c = put(id, name, value);
    Write {
        record: c.record,
        change: c.change,
        stamp: st,
        seen: Seen::new(),
        author: c.author,
    }
}

fn open<H: Harness>() -> (H, DeviceId) {
    let h = H::fresh();
    let d = ready(h.store().device()).unwrap();
    (h, d)
}

fn local<S: Store>(s: &S, pt: u64, cs: &[LocalChange]) -> Vec<Write> {
    ready(s.apply_local(pt, cs)).unwrap()
}

fn recv<S: Store>(s: &S, b: &[Write]) -> Report {
    ready(s.apply(b)).unwrap()
}

fn all<S: Store>(s: &S) -> Vec<(Position, Write)> {
    ready(s.writes_since(Position::START)).unwrap()
}

/// `Σ` observable through the port: writes with positions, plus the watermark of `peer`.
fn sigma<S: Store>(s: &S, peer: DeviceId) -> (Vec<(Position, Write)>, Option<Watermark>) {
    (all(s), ready(s.watermark(peer)).unwrap())
}

/// Round trip of a received write: `==` through `writes_of`, `writes_since`, and after reopen.
fn round_trip<H: Harness>(w: Write) {
    let (mut h, _) = open::<H>();
    for _ in 0..2 {
        if all(h.store()).is_empty() {
            recv(h.store(), std::slice::from_ref(&w));
        }
        let s = h.store();
        assert_eq!(ready(s.writes_of(&w.record)), Ok(vec![w.clone()]));
        let got: Vec<Write> = all(s).into_iter().map(|(_, w)| w).collect();
        assert_eq!(got, vec![w.clone()]);
        h.reopen();
    }
}

fn named<H: Harness>() {
    let p1 = dev(1);

    // US-5 case 1: 3 items, 1 deleted, sorted; case 12: unknown record.
    let (h, _) = open::<H>();
    let s = h.store();
    local(
        s,
        1,
        &[
            put("c", "f", b"3"),
            put("a", "f", b"1"),
            put("b", "f", b"2"),
            del("b"),
        ],
    );
    let recs = ready(s.records("m", "g")).unwrap();
    assert_eq!(
        recs.iter().map(|(r, _)| r.id.as_str()).collect::<Vec<_>>(),
        ["a", "b", "c"]
    );
    let deleted: Vec<_> = recs
        .iter()
        .filter(|(_, st)| matches!(st.status, Status::Deleted(_)))
        .map(|(r, _)| r.id.as_str())
        .collect();
    assert_eq!(deleted, ["b"]);
    assert_eq!(ready(s.state(&rid("nope"))), Ok(None));
    assert_eq!(ready(s.writes_of(&rid("nope"))), Ok(vec![]));

    // US-5 case 2: record with only a deletion.
    let (h, _) = open::<H>();
    local(h.store(), 1, &[del("z")]);
    let st = ready(h.store().state(&rid("z"))).unwrap().unwrap();
    assert!(matches!(st.status, Status::Deleted(_)) && st.fields.is_empty());

    // US-5 cases 3, 5: 10,000 writes on one field, `writes_since(⊥P)` in position order.
    let (h, _) = open::<H>();
    let s = h.store();
    for _ in 0..20 {
        let cs: Vec<_> = (0..500).map(|i| put("r", "f", &[i as u8])).collect();
        local(s, 1, &cs);
    }
    assert_eq!(ready(s.writes_of(&rid("r"))).unwrap().len(), 10_000);
    let a = all(s);
    assert_eq!(a.len(), 10_000);
    assert!(a.windows(2).all(|w| w[0].0 < w[1].0));
    assert_eq!(ready(s.writes_since(Position(u64::MAX))), Ok(vec![]));

    // US-5 case 6: late write with an old stamp is returned after `P_before`.
    let (h, _) = open::<H>();
    let s = h.store();
    recv(s, &[wr("r", "f", b"new", stamp(100, 0, p1))]);
    let before = all(s).last().unwrap().0;
    let late = wr("r", "g", b"old", stamp(1, 0, p1));
    recv(s, std::slice::from_ref(&late));
    let tail = ready(s.writes_since(before)).unwrap();
    assert_eq!(
        tail.into_iter().map(|(_, w)| w).collect::<Vec<_>>(),
        vec![late]
    );

    // US-5 case 8: same positions after reopen.
    let (mut h, _) = open::<H>();
    local(h.store(), 1, &[put("a", "f", b"1"), put("b", "f", b"2")]);
    let before = all(h.store());
    h.reopen();
    assert_eq!(all(h.store()), before);
    let from = before[0].0;
    assert_eq!(ready(h.store().writes_since(from)).unwrap(), before[1..]);

    // US-5 cases 9, 10: unknown watermark; lower and equal marks leave it unchanged.
    let (h, _) = open::<H>();
    let s = h.store();
    assert_eq!(ready(s.watermark(p1)), Ok(None));
    ready(s.advance_watermark(p1, Watermark(5, 2))).unwrap();
    for to in [Watermark(5, 1), Watermark(4, 9), Watermark(5, 2)] {
        ready(s.advance_watermark(p1, to)).unwrap();
        assert_eq!(ready(s.watermark(p1)), Ok(Some(Watermark(5, 2))));
    }

    // US-1 case 6: unusual write (empty value, NUL and emoji, 3 `seen` entries, `c = u32::MAX - 1`).
    let mut w = wr("a\0😀", "f\0é", b"", stamp(7, u32::MAX - 1, p1));
    w.record = rid_in("m\0", "g😀", "a\0😀");
    w.seen = BTreeMap::from([
        (TargetKind::Delete, stamp(1, 0, p1)),
        (TargetKind::Field(Field("f\0é".into())), stamp(2, 3, p1)),
        (TargetKind::Field(Field("g".into())), stamp(7, 0, p1)),
    ]);
    round_trip::<H>(w);

    // US-1 case 6b (DA F-1): `seen` with `Field("")` and a `Delete` entry.
    let mut w = wr("a", "f", b"v", stamp(5, 0, p1));
    w.seen = BTreeMap::from([
        (TargetKind::Field(Field(String::new())), stamp(1, 0, p1)),
        (TargetKind::Delete, stamp(2, 0, p1)),
    ]);
    round_trip::<H>(w);

    // US-1 case 8: clock went back; `K = (10⁹, 3)`, `pt = 0` after reopen.
    let (mut h, d) = open::<H>();
    for _ in 0..4 {
        local(h.store(), 1_000_000_000, &[put("a", "f", b"x")]);
    }
    h.reopen();
    let w = local(h.store(), 0, &[put("a", "f", b"y")]);
    assert_eq!(w[0].stamp, stamp(1_000_000_000, 4, d));

    // US-1 case 9: received stamp far ahead; the next local stamp is above it.
    let (h, d) = open::<H>();
    let far = wr("a", "f", b"x", stamp(1_000_000_000_000, 0, p1));
    recv(h.store(), std::slice::from_ref(&far));
    let w = local(h.store(), 5, &[put("a", "f", b"y")]);
    assert_eq!(w[0].stamp, stamp(1_000_000_000_000, 1, d));

    // DA F-2: received write at exactly the new `K` with `a > D`, then reads succeed.
    let (mut h, d) = open::<H>();
    let hi = wr("r", "f", b"x", stamp(20, 3, dev(u128::MAX)));
    recv(h.store(), std::slice::from_ref(&hi));
    for _ in 0..2 {
        assert_eq!(ready(h.store().writes_of(&hi.record)), Ok(vec![hi.clone()]));
        assert!(ready(h.store().state(&hi.record)).unwrap().is_some());
        h.reopen();
    }
    assert_eq!(
        local(h.store(), 0, &[put("r", "f", b"y")])[0].stamp,
        stamp(20, 4, d)
    );

    // Q-A on the adapter (DA F-3): stored local identity re-sent with another content.
    let (mut h, d) = open::<H>();
    let s = h.store();
    let stored = local(s, 10, &[put("r", "f", b"a")]);
    ready(s.advance_watermark(p1, Watermark(1, 0))).unwrap();
    let before = sigma(s, p1);
    let mut reused = stored[0].clone();
    reused.change = put("r", "f", b"b").change;
    let batch = [wr("s", "f", b"x", stamp(99, 0, p1)), reused];
    assert_eq!(
        ready(s.apply(&batch)),
        Err(ApplyError::Rejected(Rejection::LocalIdentityReused {
            index: 1
        }))
    );
    assert_eq!(sigma(s, p1), before);
    assert_eq!(ready(s.state(&rid("s"))), Ok(None));
    // `K` unchanged (the peer write at `l = 99` was not adopted); the handle still works.
    assert_eq!(
        local(s, 0, &[put("r", "f", b"c")])[0].stamp,
        stamp(10, 1, d)
    );
    h.reopen();
    assert_eq!(all(h.store()).len(), 2);

    // Q-A: unknown local identity received is accepted and raises `K`.
    let (h, d) = open::<H>();
    recv(h.store(), &[wr("r", "f", b"x", stamp(50, 0, d))]);
    assert_eq!(
        local(h.store(), 0, &[put("r", "f", b"y")])[0].stamp,
        stamp(50, 1, d)
    );
}
