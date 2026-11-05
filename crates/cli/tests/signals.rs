//! Signals shown once at the next action (ST-04 E-ST04-6; matrix US-6; DA F-13).

mod support;

use std::path::{Path, PathBuf};

use support::*;

fn setup(tag: &str) -> PathBuf {
    let store = tmp(tag).join("arbitre.db");
    let r = run(&mut on(
        &store,
        &[],
        &["module", "enable", "--module", "fake-a"],
    ));
    assert_eq!(r.code, 0, "{}", r.stderr);
    store
}

fn signal(store: &Path, source: &str, message: &str) {
    let mut c = on(
        store,
        &[],
        &["test", "signal", "--source", source, "--message", message],
    );
    assert_eq!(run(&mut c).code, 0);
}

fn shown(r: &Run, message: &str) -> usize {
    r.stderr
        .lines()
        .filter(|l| l.starts_with("signal (") && l.ends_with(message))
        .count()
}

#[test]
fn shown_once_on_stderr() {
    // Matrix US-6 rows 1-2.
    let store = setup("sig-once");
    signal(&store, "calendar_refresh", "calendar not refreshed");
    let first = run(&mut on(&store, &[], &["fake-a", "list", "--json"]));
    assert_eq!(first.code, 0);
    assert_eq!(
        shown(&first, "calendar not refreshed"),
        1,
        "{}",
        first.stderr
    );
    assert!(
        !first.stdout.contains("calendar"),
        "not in the result document"
    );
    doc(&first);
    let second = run(&mut on(&store, &[], &["fake-a", "list"]));
    assert_eq!(
        shown(&second, "calendar not refreshed"),
        0,
        "{}",
        second.stderr
    );
}

#[test]
fn not_shown_on_help_nor_usage() {
    // Matrix US-6 row 3.
    let store = setup("sig-help");
    signal(&store, "sync", "sync failed");
    for args in [&["--help"][..], &["fake-a", "list", "--bogus"]] {
        let r = run(&mut on(&store, &[], args));
        assert_eq!(shown(&r, "sync failed"), 0, "{args:?}");
    }
    let r = run(&mut on(&store, &[], &["fake-a", "list"]));
    assert_eq!(shown(&r, "sync failed"), 1, "{}", r.stderr);
}

#[test]
fn oldest_first() {
    // Matrix US-6 row 5.
    let store = setup("sig-order");
    let at = |t: &str, msg: &str| {
        let mut c = on(
            &store,
            &[],
            &["test", "signal", "--source", "s", "--message", msg],
        );
        assert_eq!(run(c.env("ARBITRE_TEST_NOW", t)).code, 0);
    };
    at("2000", "second");
    at("1000", "first");
    let r = run(&mut on(&store, &[], &["fake-a", "list"]));
    let lines: Vec<_> = r
        .stderr
        .lines()
        .filter(|l| l.starts_with("signal ("))
        .collect();
    assert_eq!(lines.len(), 2, "{}", r.stderr);
    assert!(
        lines[0].ends_with("first") && lines[1].ends_with("second"),
        "{lines:?}"
    );
}

#[test]
fn local_time_from_tz() {
    // DA F-13: the native clock's time zone and pinned time (2026-01-15T14:05:00Z).
    let store = setup("sig-tz");
    let mut c = on(
        &store,
        &[],
        &["test", "signal", "--source", "s", "--message", "tz"],
    );
    assert_eq!(run(c.env("ARBITRE_TEST_NOW", "1768485900000")).code, 0);
    let r = run(on(&store, &[], &["fake-a", "list"]).env("TZ", "America/New_York"));
    assert!(
        r.stderr.contains("signal (2026-01-15 09:05): tz"),
        "{}",
        r.stderr
    );
}

#[test]
fn source_must_be_snake_case() {
    // RG-ST04-6.1: `source` is part of the record id.
    let store = setup("sig-source");
    let before = snapshot(&store);
    for bad in ["", "Calendar", "1sync", "sync-x", "sync x", "a__b", "sync_"] {
        let args = [
            "test",
            "signal",
            "--source",
            bad,
            "--message",
            "m",
            "--json",
        ];
        let (_, code, id) = failure(&doc(&run(&mut on(&store, &[], &args))));
        assert_eq!((code, id.as_str()), (3, "internal_error"), "{bad:?}");
    }
    assert_eq!(snapshot(&store), before, "nothing written");
    signal(&store, "sync_2", "ok");
}
