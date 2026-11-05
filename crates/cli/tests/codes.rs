//! One exit code per category, crash, nothing changes on failure, storage ids, unwritable
//! output (ST-03 E-ST03-3, E-ST03-4; matrix US-3; RG-ST03-3.1 to 3.6, 4.5; DA F-5).

mod support;

use std::path::{Path, PathBuf};

use support::*;

fn store(home: &Path) -> PathBuf {
    home.join("arbitre.db")
}

fn on_store(home: &Path, path: &Path, args: &[&str], json: bool) -> Run {
    let s = path.display().to_string();
    let mut v: Vec<&str> = args.to_vec();
    v.extend(["--store", s.as_str()]);
    if json {
        v.push("--json");
    }
    arbitre(home, &v)
}

fn on(home: &Path, args: &[&str], json: bool) -> Run {
    on_store(home, &store(home), args, json)
}

#[test]
fn one_code_per_category_in_both_modes() {
    // Matrix US-3 rows 1-6.
    let cases: [(&[&str], i32, Option<&str>); 5] = [
        (&["test", "noop"], 0, None),
        (&["test", "refuse"], 1, Some("refusal")),
        (&["test", "echo", "--bogus"], 2, Some("usage")),
        (&["test", "fail"], 3, Some("tool_error")),
        (&["test", "not-enabled"], 4, Some("module_not_enabled")),
    ];
    for (args, code, category) in cases {
        let home = tmp("codes-table");
        let r = on(&home, args, false);
        assert_eq!(r.code, code, "{args:?}: {}", r.stderr);
        if code != 0 {
            assert_eq!(r.stdout, "", "{args:?}");
        }
        let r = on(&home, args, true);
        assert_eq!(r.code, code, "{args:?} --json: {}", r.stderr);
        let d = doc(&r);
        match category {
            None => {
                success(&d);
            }
            Some(c) => {
                let (got, n, _) = failure(&d);
                assert_eq!((got.as_str(), n), (c, i64::from(code)), "{args:?}");
            }
        }
    }
}

#[test]
fn crash_is_internal_error() {
    // Matrix US-3 row 7, RG-ST03-3.5.
    let home = tmp("codes-panic");
    let r = on(&home, &["test", "panic"], false);
    assert_eq!(r.code, 3);
    assert_eq!(r.stdout, "");
    assert!(!r.stderr.is_empty());
    let r = on(&home, &["test", "panic"], true);
    assert_eq!(r.code, 3);
    let (c, n, id) = failure(&doc(&r));
    assert_eq!(
        (c.as_str(), n, id.as_str()),
        ("tool_error", 3, "internal_error")
    );
}

#[test]
fn output_mismatch() {
    // Matrix US-4 row 6, RG-ST03-4.5.
    let home = tmp("codes-bad-output");
    let r = on(&home, &["test", "bad-output"], true);
    assert_eq!(r.code, 3);
    assert_eq!(failure(&doc(&r)).2, "output_mismatch");
    let r = on(&home, &["test", "bad-output"], false);
    assert_eq!((r.code, r.stdout.as_str()), (3, ""));
}

#[test]
fn write_succeeds_and_changes_the_store() {
    let home = tmp("codes-write");
    let path = store(&home);
    populate(&path);
    let before = snapshot(&path).0.len();
    let r = on(&home, &["test", "write", "--key", "k"], true);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(success(&doc(&r))["key"], "k");
    assert_eq!(snapshot(&path).0.len(), before + 1);
}

#[test]
fn failed_mutating_action_changes_nothing() {
    // E-ST03-4, matrix US-3 row 10, PROP-ST03-1.
    for (mode, code) in [("refusal", 1), ("tool-error", 3), ("panic", 3)] {
        for json in [false, true] {
            let home = tmp("codes-fail-write");
            let path = store(&home);
            populate(&path);
            let before = snapshot(&path);
            let r = on(
                &home,
                &["test", "write", "--key", "k", "--fail", mode],
                json,
            );
            assert_eq!(r.code, code, "{mode} json={json}: {}", r.stderr);
            if json {
                assert_eq!(failure(&doc(&r)).1, i64::from(code));
            }
            assert_eq!(snapshot(&path), before, "{mode} json={json}");
            let _ = std::fs::remove_dir_all(home);
        }
    }
}

#[test]
fn damaged_store_is_corrupted_and_untouched() {
    // Matrix US-3 row 9, RG-ST03-5.3.
    let home = tmp("codes-damaged");
    let path = store(&home);
    std::fs::write(&path, b"not a store\n").unwrap();
    let r = on(&home, &["test", "read"], true);
    assert_eq!(r.code, 3);
    assert_eq!(failure(&doc(&r)).2, "storage_corrupted");
    assert_eq!(std::fs::read(&path).unwrap(), b"not a store\n");
}

#[test]
fn newer_layout_is_incompatible_and_untouched() {
    // Matrix US-3 row 9: `user_version` (header offset 60) above the supported one.
    let home = tmp("codes-newer");
    let path = store(&home);
    populate(&path);
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[60..64].copy_from_slice(&99u32.to_be_bytes());
    std::fs::write(&path, &bytes).unwrap();
    let r = on(&home, &["test", "read"], true);
    assert_eq!(r.code, 3);
    assert_eq!(failure(&doc(&r)).2, "storage_incompatible_version");
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}

#[test]
fn unwritable_stdout_is_output_failed() {
    // DA F-5: read end closed before spawn; exit 3 in both modes, never 0 or 101.
    for json in [false, true] {
        let home = tmp("codes-pipe");
        let s = store(&home).display().to_string();
        let mut args = vec!["test", "echo", "--text", "hi", "--store", s.as_str()];
        if json {
            args.push("--json");
        }
        let (reader, writer) = std::io::pipe().unwrap();
        drop(reader);
        let o = cmd(&home, &args).stdout(writer).output().expect("spawn");
        assert_eq!(o.status.code(), Some(3), "json={json}");
        assert!(!o.stderr.is_empty(), "json={json}");
    }
}
