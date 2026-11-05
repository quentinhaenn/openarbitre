//! Enable, disable, list; invisible and not enabled; isolation; revalidation on enable
//! (ST-04 E-ST04-1, 2, 3, 7; matrices US-1, US-2, US-3, US-7).

mod support;

use std::path::{Path, PathBuf};

use support::*;

struct Dev {
    home: PathBuf,
    store: PathBuf,
}

fn dev(tag: &str) -> Dev {
    let home = tmp(tag);
    let store = home.join("arbitre.db");
    Dev { home, store }
}

impl Dev {
    fn run(&self, args: &[&str], json: bool) -> Run {
        let s = self.store.display().to_string();
        let mut v = args.to_vec();
        v.extend(["--store", s.as_str()]);
        if json {
            v.push("--json");
        }
        arbitre(&self.home, &v)
    }

    #[track_caller]
    fn ok(&self, args: &[&str]) -> serde_json::Value {
        let r = self.run(args, true);
        assert_eq!(r.code, 0, "{args:?}: {}", r.stdout);
        success(&doc(&r))
    }

    fn enable(&self, m: &str) -> serde_json::Value {
        self.ok(&["module", "enable", "--module", m])
    }

    fn disable(&self, m: &str) -> serde_json::Value {
        self.ok(&["module", "disable", "--module", m])
    }
}

fn bytes(p: &Path) -> Option<Vec<u8>> {
    std::fs::read(p).ok()
}

#[test]
fn enable_disable_idempotent_and_listed() {
    // Matrix US-1 rows 1, 2, 5.
    let d = dev("mod-enable");
    let e = d.enable("fake-a");
    assert_eq!(
        (e["enabled"].clone(), e["changed"].clone()),
        (true.into(), true.into())
    );
    assert_eq!(e["problems"], serde_json::json!([]));
    let before = snapshot(&d.store);
    assert_eq!(d.enable("fake-a")["changed"], false);
    assert_eq!(snapshot(&d.store), before, "second enable writes nothing");
    let list = d.ok(&["module", "list"]);
    let m = |id: &str| {
        list["modules"]
            .as_array()
            .expect("modules")
            .iter()
            .find(|m| m["id"] == id)
            .unwrap_or_else(|| panic!("{id} listed"))
            .clone()
    };
    assert_eq!(m("fake-a")["enabled"], true);
    assert!(
        !m("fake-a")["actions"]
            .as_array()
            .expect("actions")
            .is_empty()
    );
    assert_eq!(m("fake-b")["enabled"], false);
    assert_eq!(m("fake-b")["actions"], serde_json::json!([]));
    assert_eq!(d.disable("fake-a")["changed"], true);
    assert_eq!(d.disable("fake-a")["changed"], false);
}

#[test]
fn disabling_keeps_the_data() {
    // Matrix US-1 row 3 (RG-ST04-1.3).
    let d = dev("mod-keep");
    d.enable("fake-a");
    d.ok(&["fake-a", "put", "--key", "k", "--value", "v"]);
    d.disable("fake-a");
    d.enable("fake-a");
    assert_eq!(d.ok(&["fake-a", "get", "--key", "k"])["value"], "v");
}

#[test]
fn unknown_or_foundation_module_is_usage() {
    // Matrix US-1 rows 6, 7.
    let d = dev("mod-unknown");
    for m in ["nope", "arbitre"] {
        let r = d.run(&["module", "enable", "--module", m], true);
        assert_eq!(r.code, 2, "{m}");
    }
    let r = d.run(&["module", "disable", "--module", "arbitre"], false);
    assert_eq!(r.code, 2);
    assert_eq!(bytes(&d.store), None, "store not created");
}

#[test]
fn fresh_device_shows_only_the_foundation() {
    // Matrix US-1 row 8; US-2 row 5.
    let d = dev("mod-fresh");
    let r = d.run(&["--help"], false);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("module"), "{}", r.stdout);
    assert!(!r.stdout.contains("fake-a"), "{}", r.stdout);
    assert_eq!(bytes(&d.store), None, "store not created");
}

#[test]
fn ci_flow_enable_then_run() {
    // Matrix US-1 row 9 (archi D-2, Q7 (a)).
    let home = tmp("mod-ci");
    let s = home.join("ci.db");
    let r = run(cmd(&home, &["module", "enable", "--module", "fake-a"]).env("ARBITRE_STORE", &s));
    assert_eq!(r.code, 0, "{}", r.stderr);
    let r = run(cmd(&home, &["fake-a", "list"]).env("ARBITRE_STORE", &s));
    assert_eq!(r.code, 0, "{}", r.stderr);
}

#[test]
fn disabled_module_is_hidden_and_refused() {
    // Matrix US-2 rows 1-4, 8.
    let d = dev("mod-hidden");
    populate(&d.store);
    let before = snapshot(&d.store);
    let r = d.run(&["--help"], false);
    assert!(!r.stdout.contains("fake-a"), "{}", r.stdout);
    for args in [
        &["fake-a", "--help"][..],
        &["fake-a", "put", "--key", "k", "--value", "v"],
        &["fake-a", "put", "--bogus"],
    ] {
        let r = d.run(args, false);
        assert_eq!(r.code, 4, "{args:?}: {}", r.stderr);
        assert!(
            r.stderr.contains("module enable --module fake-a"),
            "{}",
            r.stderr
        );
        let r = d.run(args, true);
        let (cat, code, id) = failure(&doc(&r));
        assert_eq!(
            (cat.as_str(), code, id.as_str()),
            ("module_not_enabled", 4, "module_not_enabled")
        );
    }
    assert_eq!(snapshot(&d.store), before, "nothing written");
    d.enable("fake-a");
    assert_eq!(d.run(&["fake-a", "list"], false).code, 0);
}

#[test]
fn not_enabled_never_creates_the_store() {
    // Matrix US-2 rows 5, 6.
    let d = dev("mod-absent");
    let s = d.store.display().to_string();
    for args in [
        vec!["fake-a", "put", "--bogus", "--store", s.as_str()],
        vec!["--json", "--store", s.as_str(), "fake-a", "put"],
    ] {
        assert_eq!(arbitre(&d.home, &args).code, 4, "{args:?}");
    }
    assert_eq!(bytes(&d.store), None);
}

#[test]
fn non_store_files_are_never_written() {
    // Matrix US-2 row 7; DA F-5: text, 1 byte, an SQLite file with no table.
    let d = dev("mod-damaged");
    let sqlite_no_table: &[u8] = b"SQLite format 3\0";
    for content in [&b"not a store"[..], b"x", sqlite_no_table] {
        std::fs::write(&d.store, content).expect("write");
        let r = d.run(&["--help"], true);
        let (_, code, id) = failure(&doc(&r));
        assert_eq!((code, id.as_str()), (3, "storage_corrupted"), "{content:?}");
        assert_eq!(bytes(&d.store).as_deref(), Some(content), "file untouched");
        assert_eq!(d.run(&["--version"], false).code, 0);
    }
}

#[test]
fn store_option_after_the_word_is_the_store_read() {
    // DA F-6: the enabled state comes from `--store` wherever it is.
    let d = dev("mod-store-after");
    d.enable("fake-a");
    let s = d.store.display().to_string();
    let r = arbitre(&d.home, &["fake-a", "list", "--store", s.as_str()]);
    assert_eq!(r.code, 0, "{}", r.stderr);
}

#[test]
fn modules_see_only_their_own_data() {
    // Matrix US-3 rows 1-2.
    let d = dev("mod-isolation");
    d.enable("fake-a");
    d.enable("fake-b");
    assert!(
        d.ok(&["fake-a", "list"])["entries"]
            .as_array()
            .expect("list")
            .is_empty()
    );
    d.ok(&["fake-b", "put", "--key", "k", "--value", "2"]);
    assert!(
        d.ok(&["fake-a", "list"])["entries"]
            .as_array()
            .expect("list")
            .is_empty()
    );
    d.ok(&["fake-a", "put", "--key", "k", "--value", "1"]);
    assert_eq!(d.ok(&["fake-a", "get", "--key", "k"])["value"], "1");
    assert_eq!(d.ok(&["fake-b", "get", "--key", "k"])["value"], "2");
}

#[test]
fn enable_reports_invalid_records() {
    // Matrix US-7 rows 1-3.
    let d = dev("mod-revalidate");
    d.enable("fake-a");
    d.ok(&["fake-a", "put", "--key", "k", "--value", "invalid"]);
    d.ok(&["fake-a", "put", "--key", "j", "--value", "fine"]);
    d.disable("fake-a");
    let e = d.enable("fake-a");
    assert_eq!(
        e["problems"],
        serde_json::json!([{
            "category": "entry", "id": "k", "problem": "invalid_value",
            "message": e["problems"][0]["message"].clone()
        }])
    );
    assert_eq!(d.enable("fake-a")["problems"], serde_json::json!([]));
}

#[test]
fn relay_action_fails_explicitly() {
    // Matrix US-4 row 5 (no relay is ever configured in ST-04).
    let d = dev("mod-push");
    d.enable("fake-a");
    let r = d.run(&["fake-a", "push"], true);
    let (_, code, id) = failure(&doc(&r));
    assert_eq!((code, id.as_str()), (3, "relay_not_configured"));
}
