//! Re-exec protocol shared by the process harnesses (plan T-10, T-11; read by T-13).
//!
//! A test target declares `#[test] fn child_entry() { child::child_main() }`. The parent
//! re-runs its own binary with `--exact child_entry` and `ARBITRE_CHILD_ROLE`; the child talks
//! over stdin (command lines, EOF = stop) and stdout (lines prefixed `@ `; others are libtest
//! chatter). Env: `ARBITRE_CHILD_PATH` (store), `ARBITRE_CHILD_WAIT_MS` (busy wait, default
//! 5000), `ARBITRE_CHILD_ID`, `ARBITRE_CHILD_SEED`, `ARBITRE_CHILD_K`.
//!
//! Roles and their lines (child -> parent):
//! - `open`: `ready`; waits `go`; `opened <hex D>` | `open_failed <dbg>`.
//! - `writer`: `opened`; per batch `start <b> <n>` then `confirmed <b> <dbg Vec<Write>>`
//!   (or `failed <b> <dbg>`); stops on any stdin line or EOF. Record ids are `<id>.<b>.<i>`.
//! - `steps`: `opened`; commands `local <pt> <id=field=value|id=-;...>` -> `ok <dbg>` | `err
//!   <dbg>`; `hold <ms>` -> `held`, `released`; `since` -> `since <dbg>`.
//! - `free_writer` / `free_reader`: free-running phase (RG-ST02-3.5), see `interleave.rs`.
#![allow(dead_code)]

use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display};
use std::future::Future;
use std::io::{BufRead, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TryRecvError};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use arbitre_core::{
    Author, Change, DeviceId, Field, FieldValue, LocalChange, Position, RecordId, Seen, Stamp,
    Store, Write,
};
use arbitre_sqlite::{Options, SqliteStore};

const PREFIX: &str = "@ ";

/// splitmix64; the base seed comes from `ARBITRE_SEED` (printed by the tests).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    pub fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

pub fn seed() -> u64 {
    std::env::var("ARBITRE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x5EED)
}

pub fn hex(d: DeviceId) -> String {
    d.0.iter().map(|b| format!("{b:02x}")).collect()
}

/// Polls a store future once (RG-ST02-6.3).
pub fn block_on<F: Future>(f: F) -> F::Output {
    match std::pin::pin!(f).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("store future not ready on first poll"),
    }
}

pub fn rec(category: &str, id: &str) -> RecordId {
    RecordId {
        module: "m".into(),
        category: category.into(),
        id: id.into(),
    }
}

pub fn field_change(field: &str, value: &[u8]) -> Change {
    Change::Field {
        name: Field(field.into()),
        value: FieldValue(value.to_vec()),
    }
}

/// Fresh empty directory; the caller removes it.
pub fn fresh_dir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "arbitre-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------- parent side

pub struct Child {
    proc: std::process::Child,
    stdin: Option<std::process::ChildStdin>,
    lines: Receiver<String>,
    /// Every protocol line received so far.
    pub log: Vec<String>,
}

impl Child {
    pub fn spawn(role: &str, path: &Path, env: &[(&str, String)]) -> Self {
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args(["child_entry", "--exact", "--nocapture", "--test-threads=1"])
            .env("ARBITRE_CHILD_ROLE", role)
            .env("ARBITRE_CHILD_PATH", path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut proc = cmd.spawn().expect("spawn child");
        let stdin = proc.stdin.take();
        let mut out = BufReader::new(proc.stdout.take().unwrap());
        let (tx, lines) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = String::new();
            loop {
                buf.clear();
                // A line cut by a kill has no newline: not a confirmation.
                if !matches!(out.read_line(&mut buf), Ok(n) if n > 0) || !buf.ends_with('\n') {
                    break;
                }
                // libtest's "test x ... " header can precede the first line on its row.
                if let Some((_, l)) = buf.split_once(PREFIX)
                    && tx.send(l.trim_end().to_string()).is_err()
                {
                    break;
                }
            }
        });
        Self {
            proc,
            stdin,
            lines,
            log: Vec::new(),
        }
    }

    pub fn send(&mut self, line: &str) {
        if let Some(s) = self.stdin.as_mut() {
            let _ = writeln!(s, "{line}");
            let _ = s.flush();
        }
    }

    pub fn next(&mut self, timeout: Duration) -> Option<String> {
        match self.lines.recv_timeout(timeout) {
            Ok(l) => {
                self.log.push(l.clone());
                Some(l)
            }
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => None,
        }
    }

    /// Next line starting with `prefix` (earlier lines stay in `log`); panics on timeout or EOF.
    pub fn expect(&mut self, prefix: &str, timeout: Duration) -> String {
        loop {
            match self.next(timeout) {
                Some(l) if l.starts_with(prefix) => return l,
                Some(_) => {}
                None => panic!("no `{prefix}` line from child; log: {:?}", self.log),
            }
        }
    }

    /// Closes stdin (the child stops at its next check).
    pub fn close(&mut self) {
        self.stdin = None;
    }

    /// Forced stop (SIGKILL).
    pub fn kill(&mut self) {
        let _ = self.proc.kill();
    }

    /// Waits for exit and returns the whole protocol log.
    pub fn finish(&mut self) -> Vec<String> {
        let _ = self.proc.wait();
        while let Ok(l) = self.lines.recv() {
            self.log.push(l);
        }
        self.log.clone()
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

// ----------------------------------------------------------------- child side

fn say(line: impl Display) {
    println!("{PREFIX}{line}");
}

fn dbg(x: &impl Debug) -> String {
    format!("{x:?}")
}

fn open_or_exit(path: &Path, wait: Duration) -> SqliteStore {
    let options = Options {
        wait,
        ..Options::default()
    };
    match SqliteStore::open(path, &options) {
        Ok(s) => {
            say(format!("opened {}", hex(block_on(s.device()).unwrap())));
            s
        }
        Err(e) => {
            say(format!("open_failed {}", dbg(&e)));
            std::process::exit(1);
        }
    }
}

fn stopped(rx: &Receiver<String>) -> bool {
    matches!(rx.try_recv(), Ok(_) | Err(TryRecvError::Disconnected))
}

fn parse_changes(spec: &str) -> Vec<LocalChange> {
    spec.split(';')
        .map(|item| {
            let parts: Vec<&str> = item.split('=').collect();
            let change = match parts[..] {
                [_, "-"] => Change::Delete,
                [_, f, v] => field_change(f, v.as_bytes()),
                _ => panic!("bad spec {item}"),
            };
            LocalChange {
                record: rec("c", parts[0]),
                change,
                author: Author::CliUser,
            }
        })
        .collect()
}

/// Entry of the re-executed test binary; returns at once when not a child.
pub fn child_main() {
    let Ok(role) = std::env::var("ARBITRE_CHILD_ROLE") else {
        return;
    };
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    let path = PathBuf::from(var("ARBITRE_CHILD_PATH"));
    let wait = Duration::from_millis(var("ARBITRE_CHILD_WAIT_MS").parse().unwrap_or(5000));
    let num = |k: &str| var(k).parse::<u64>().unwrap_or(0);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    match role.as_str() {
        "open" => {
            say("ready");
            if rx.recv().is_ok() {
                open_or_exit(&path, wait);
                while rx.recv().is_ok() {}
            }
        }
        "writer" => writer(
            &path,
            wait,
            &rx,
            num("ARBITRE_CHILD_ID"),
            num("ARBITRE_CHILD_SEED"),
        ),
        "steps" => steps(&path, wait, &rx),
        "free_writer" => free_writer(&path, wait, &rx, num("ARBITRE_CHILD_K") as usize),
        "free_reader" => free_reader(&path, wait, &rx, num("ARBITRE_CHILD_K") as usize),
        other => panic!("unknown role {other}"),
    }
    std::process::exit(0);
}

/// Batches of 1-5 writes, alternately local and received (foreign device `0x100 + id`).
fn writer(path: &Path, wait: Duration, rx: &Receiver<String>, id: u64, seed: u64) {
    let store = open_or_exit(path, wait);
    let mut rng = Rng::new(seed);
    for b in 0u64.. {
        if stopped(rx) {
            return;
        }
        // A tight loop would starve the other writer's busy handler (backoff sleeps).
        std::thread::sleep(Duration::from_micros(rng.below(4000)));
        let n = 1 + rng.below(5) as usize;
        let name = |i: usize| format!("{id}.{b}.{}", i % 3);
        let change = |rng: &mut Rng, i: usize| {
            if rng.chance(20) {
                Change::Delete
            } else {
                field_change("f", format!("v{id}.{b}.{i}").as_bytes())
            }
        };
        say(format!("start {b} {n}"));
        let result = if rng.chance(50) {
            let changes: Vec<LocalChange> = (0..n)
                .map(|i| LocalChange {
                    record: rec("c", &name(i)),
                    change: change(&mut rng, i),
                    author: Author::UiUser,
                })
                .collect();
            block_on(store.apply_local(rng.below(5), &changes)).map(|w| dbg(&w))
        } else {
            let batch: Vec<Write> = (0..n)
                .map(|i| Write {
                    record: rec("c", &name(i)),
                    change: change(&mut rng, i),
                    stamp: Stamp {
                        l: 1000 + b * 10 + i as u64,
                        c: 0,
                        a: DeviceId::from_u128(0x100 + u128::from(id)),
                    },
                    seen: Seen::new(),
                    author: Author::System,
                })
                .collect();
            block_on(store.apply(&batch)).map(|_| dbg(&batch))
        };
        match result {
            Ok(w) => say(format!("confirmed {b} {w}")),
            Err(e) => {
                say(format!("failed {b} {}", dbg(&e)));
                std::process::exit(1);
            }
        }
    }
}

fn steps(path: &Path, wait: Duration, rx: &Receiver<String>) {
    let store = open_or_exit(path, wait);
    while let Ok(line) = rx.recv() {
        let (cmd, rest) = line.split_once(' ').unwrap_or((line.as_str(), ""));
        match cmd {
            "local" => {
                let (pt, spec) = rest.split_once(' ').unwrap();
                match block_on(store.apply_local(pt.parse().unwrap(), &parse_changes(spec))) {
                    Ok(w) => say(format!("ok {}", dbg(&w))),
                    Err(e) => say(format!("err {}", dbg(&e))),
                }
            }
            "hold" => {
                let raw = rusqlite::Connection::open(path).unwrap();
                raw.busy_timeout(Duration::from_secs(5)).unwrap();
                raw.execute_batch("BEGIN IMMEDIATE").unwrap();
                say("held");
                std::thread::sleep(Duration::from_millis(rest.parse().unwrap()));
                raw.execute_batch("ROLLBACK").unwrap();
                say("released");
            }
            "since" => say(format!(
                "since {}",
                dbg(&block_on(store.writes_since(Position(0))).unwrap())
            )),
            other => panic!("unknown command {other}"),
        }
    }
}

/// Batch `t`: field `v` = `t` on records `r1..rk` of category `cat`; one `w` line per write.
fn free_writer(path: &Path, wait: Duration, rx: &Receiver<String>, k: usize) {
    let store = open_or_exit(path, wait);
    let mut t = 0u64;
    while !stopped(rx) {
        let changes: Vec<LocalChange> = (1..=k)
            .map(|i| LocalChange {
                record: rec("cat", &format!("r{i}")),
                change: field_change("v", t.to_string().as_bytes()),
                author: Author::System,
            })
            .collect();
        match block_on(store.apply_local(0, &changes)) {
            Ok(ws) => ws.iter().for_each(|w| say(format!("w {}", dbg(w)))),
            Err(e) => {
                say(format!("failed {t} {}", dbg(&e)));
                std::process::exit(1);
            }
        }
        t += 1;
    }
    say(format!("done {t}"));
}

/// Loops `writes_since(START)` and `records`; prints each distinct write once, and `bad` lines.
fn free_reader(path: &Path, wait: Duration, rx: &Receiver<String>, k: usize) {
    let store = open_or_exit(path, wait);
    let (mut reads, mut printed) = (0u64, HashSet::new());
    while !stopped(rx) {
        let since = block_on(store.writes_since(Position(0))).unwrap();
        let mut counts = BTreeMap::new();
        for (_, w) in &since {
            if let Change::Field { value, .. } = &w.change {
                *counts.entry(value.0.clone()).or_insert(0usize) += 1;
            }
            let d = dbg(w);
            if printed.insert(d.clone()) {
                say(format!("w {d}"));
            }
        }
        if let Some((tag, n)) = counts.iter().find(|(_, n)| **n != k) {
            say(format!("bad tag {tag:?} has {n} of {k} writes"));
        }
        let recs = block_on(store.records("m", "cat")).unwrap();
        let tags: HashSet<_> = recs
            .iter()
            .map(|(_, s)| s.fields.get(&Field("v".into())).map(|w| dbg(&w.change)))
            .collect();
        if !recs.is_empty() && (recs.len() != k || tags.len() != 1) {
            say(format!(
                "bad records {} of {k}, {} tags",
                recs.len(),
                tags.len()
            ));
        }
        reads += 1;
    }
    say(format!("done {reads}"));
}
