//! ST-04 core: module store view, pre-scan, enabled state, reserved ids, `Ai` errors
//! (E-ST04-9; specs RG-ST04-1.5, 1.6, 2.2, 3.3 to 3.5; plan T-2).
mod support;

#[path = "support/contract.rs"]
mod contract;

use std::collections::BTreeSet;

use arbitre_core::*;
use contract::ready;

const D: u128 = 7;

fn view<'a>(s: &'a SimStore, m: &'static str, c: &'a SimClock) -> ModuleStore<'a, SimStore> {
    ModuleStore::new(s, m, Author::CliUser, c)
}

fn put(id: &str, v: &str) -> (String, String, Change) {
    (
        "entry".into(),
        id.into(),
        Change::Field {
            name: Field("value".into()),
            value: FieldValue(v.as_bytes().to_vec()),
        },
    )
}

fn action(name: &str) -> ActionDescriptor {
    ActionDescriptor {
        name: name.into(),
        summary: "s".into(),
        input: Schema::Object(vec![]),
        output: Schema::Object(vec![]),
        mutating: false,
        logged_as: LoggedAs::None,
        requires_network: BTreeSet::new(),
    }
}

#[test]
fn view_writes_land_in_its_namespace_only() {
    // E-ST04-9, matrix US-3 rows 1-2.
    let s = SimStore::new(DeviceId::from_u128(D));
    let c = SimClock::new(5_000);
    let (a, b) = (view(&s, "fake-a", &c), view(&s, "fake-b", &c));
    ready(a.apply(&[put("k", "1")])).expect("apply a");
    ready(b.apply(&[put("k", "2")])).expect("apply b");
    let all = ready(s.writes_since(Position::START)).expect("writes");
    let modules: Vec<_> = all.iter().map(|(_, w)| w.record.module.as_str()).collect();
    assert_eq!(modules, ["fake-a", "fake-b"]);
    let ra = ready(a.records("entry")).expect("records a");
    assert_eq!(ra.len(), 1);
    assert_eq!(ra[0].0, "k");
    assert!(ready(a.records("other")).expect("records").is_empty());
    let wa = ready(a.writes_of("entry", "k")).expect("writes a");
    assert_eq!(wa.len(), 1);
    assert_eq!(wa[0].change, put("k", "1").2);
    assert!(ready(a.state("entry", "k")).expect("state").is_some());
    assert!(ready(a.state("entry", "nope")).expect("state").is_none());
}

#[test]
fn view_stamps_with_its_clock_and_author() {
    let s = SimStore::new(DeviceId::from_u128(D));
    let c = SimClock::new(9_000);
    let v = ModuleStore::new(&s, "fake-a", Author::Assistant, &c);
    let w = ready(v.apply(&[put("k", "1")])).expect("apply");
    assert_eq!(w[0].author, Author::Assistant);
    assert!(w[0].stamp.l >= 9_000);
    assert_eq!(ready(v.device()).expect("device"), DeviceId::from_u128(D));
    assert_eq!(v.module(), "fake-a");
}

#[test]
fn pre_scan_reads_store_and_first_word() {
    // RG-ST04-2.2, 2.3; DA F-6.
    let p = |a: &[&'static str]| pre_scan(a);
    let cases: [(&[&str], Option<&str>, Option<&str>); 10] = [
        (&[], None, None),
        (&["--json"], None, None),
        (&["fake-a", "put"], None, Some("fake-a")),
        (
            &["--json", "--store", "p", "fake-a"],
            Some("p"),
            Some("fake-a"),
        ),
        (
            &["fake-a", "list", "--store", "p"],
            Some("p"),
            Some("fake-a"),
        ),
        (&["--store=p", "fake-a"], Some("p"), Some("fake-a")),
        (&["--store", "a", "x", "--store", "b"], Some("b"), Some("x")),
        (&["--store=", "x"], Some(""), Some("x")),
        (&["--", "fake-a"], None, None),
        (&["x", "--", "--store", "p"], None, Some("x")),
    ];
    for (args, store, word) in cases {
        let got = p(args);
        assert_eq!((got.store, got.first_word), (store, word), "{args:?}");
    }
}

#[test]
fn state_record_is_per_device() {
    // RG-ST04-1.5, plan P-6.
    let r = state_record(DeviceId::from_u128(0xab));
    assert_eq!(
        (r.module.as_str(), r.category.as_str()),
        ("arbitre", "module-state")
    );
    assert_eq!(r.id, format!("{:032x}", 0xab));
}

#[test]
fn enabled_from_reads_only_ones_of_the_build() {
    // RG-ST04-1.6; DA F-15.
    let build = ["fake-a", "fake-b"];
    assert!(enabled_from(None, &build).is_empty());
    let s = SimStore::new(DeviceId::from_u128(D));
    let c = SimClock::new(1);
    let rec = state_record(DeviceId::from_u128(D));
    let raw = |m: &str, v: &[u8]| LocalChange {
        record: rec.clone(),
        change: Change::Field {
            name: Field(m.into()),
            value: FieldValue(v.to_vec()),
        },
        author: Author::CliUser,
    };
    let on = LocalChange {
        change: state_change("fake-a", true),
        ..raw("fake-a", b"")
    };
    ready(s.apply_local(c.now(), &[on, raw("fake-b", b"x"), raw("gone", b"1")])).expect("apply");
    let st = ready(s.state(&rec)).expect("state").expect("present");
    assert_eq!(enabled_from(Some(&st), &build), BTreeSet::from(["fake-a"]));
    let off = LocalChange {
        change: state_change("fake-a", false),
        ..raw("fake-a", b"")
    };
    ready(s.apply_local(c.now() + 1, &[off])).expect("apply");
    let st = ready(s.state(&rec)).expect("state").expect("present");
    assert!(enabled_from(Some(&st), &build).is_empty());
}

#[test]
fn state_change_values() {
    let f = |on| match state_change("fake-a", on) {
        Change::Field { name, value } => (name.0, value.0),
        Change::Delete => panic!("delete"),
    };
    assert_eq!(f(true), ("fake-a".to_string(), b"1".to_vec()));
    assert_eq!(f(false), ("fake-a".to_string(), b"0".to_vec()));
}

#[test]
fn catalogue_refuses_reserved_and_ill_formed_module_ids() {
    // RG-ST04-3.4; DA F-8.
    let foundation = [action("arbitre.module.list"), action("arbitre.test.echo")];
    let refused = |id: &str, needle: &str| {
        let e = Catalog::build(&foundation, &[(id, vec![])]).expect_err(id);
        assert!(e.message.contains(needle), "{id}: {}", e.message);
    };
    refused("arbitre", "foundation");
    refused("module", "reserved");
    refused("test", "reserved");
    refused("Fake", "ill-formed");
    refused("fake_a", "ill-formed");
    let c = Catalog::build(&foundation, &[("fake-a", vec![action("fake-a.put")])]).expect("valid");
    let names: Vec<_> = c.actions().iter().map(|a| a.name.as_str()).collect();
    assert_eq!(
        names,
        ["arbitre.module.list", "arbitre.test.echo", "fake-a.put"]
    );
}

#[test]
fn ai_errors_are_tool_errors_with_stable_ids() {
    // RG-ST04-3.5.
    for (e, id) in [
        (AiError::NotConfigured, "ai_not_configured"),
        (AiError::Unreachable, "ai_unreachable"),
        (AiError::KeyRefused, "ai_key_refused"),
        (AiError::Timeout, "ai_timeout"),
    ] {
        let a = ActionError::from(e);
        assert_eq!((a.category(), a.id()), (Category::ToolError, id));
    }
}
