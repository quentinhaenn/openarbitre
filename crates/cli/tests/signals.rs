//! Signals shown once at the next action (ST-04 E-ST04-6; matrix US-6; DA F-13).

mod support;

use std::path::{Path, PathBuf};

use support::*;

fn on(home: &Path, store: &Path, args: &[&str]) -> std::process::Command {
    let s = store.display().to_string();
    let mut v = args.to_vec();
    v.extend(["--store", s.as_str()]);
    cmd(home, &v)
}

fn setup(tag: &str) -> (PathBuf, PathBuf) {
    let home = tmp(tag);
    let store = home.join("arbitre.db");
    let r = run(&mut on(
        &home,
        &store,
        &["module", "enable", "--module", "fake-a"],
    ));
    assert_eq!(r.code, 0, "{}", r.stderr);
    (home, store)
}

fn signal(home: &Path, store: &Path, source: &str, message: &str) {
    let mut c = on(
        home,
        store,
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
    let (home, store) = setup("sig-once");
    signal(&home, &store, "calendar_refresh", "calendar not refreshed");
    let first = run(&mut on(&home, &store, &["fake-a", "list", "--json"]));
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
    let second = run(&mut on(&home, &store, &["fake-a", "list"]));
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
    let (home, store) = setup("sig-help");
    signal(&home, &store, "sync", "sync failed");
    for args in [&["--help"][..], &["fake-a", "list", "--bogus"]] {
        let r = run(&mut on(&home, &store, args));
        assert_eq!(shown(&r, "sync failed"), 0, "{args:?}");
    }
    let r = run(&mut on(&home, &store, &["fake-a", "list"]));
    assert_eq!(shown(&r, "sync failed"), 1, "{}", r.stderr);
}

#[test]
fn oldest_first() {
    // Matrix US-6 row 5.
    let (home, store) = setup("sig-order");
    let at = |t: &str, msg: &str| {
        let mut c = on(
            &home,
            &store,
            &["test", "signal", "--source", "s", "--message", msg],
        );
        assert_eq!(run(c.env("ARBITRE_TEST_NOW", t)).code, 0);
    };
    at("2000", "second");
    at("1000", "first");
    let r = run(&mut on(&home, &store, &["fake-a", "list"]));
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
    let (home, store) = setup("sig-tz");
    let mut c = on(
        &home,
        &store,
        &["test", "signal", "--source", "s", "--message", "tz"],
    );
    assert_eq!(run(c.env("ARBITRE_TEST_NOW", "1768485900000")).code, 0);
    let r = run(on(&home, &store, &["fake-a", "list"]).env("TZ", "America/New_York"));
    assert!(
        r.stderr.contains("signal (2026-01-15 09:05): tz"),
        "{}",
        r.stderr
    );
}
