//! Malformed commands (ST-03 E-ST03-1, matrix US-1; RG-ST03-1.1 to 1.3, INV-ST03-1).

mod support;

use std::path::Path;

use support::*;

/// From well-formed `test write --key k` / `test echo --text hi`, one defect each.
fn defects() -> Vec<(&'static str, Vec<String>)> {
    let long = "x".repeat(201);
    let cases: Vec<(&str, Vec<&str>)> = vec![
        ("missing required", vec!["test", "write"]),
        (
            "unknown option",
            vec!["test", "write", "--key", "k", "--bogus"],
        ),
        ("unknown word", vec!["test", "nope", "--key", "k"]),
        ("extra word", vec!["test", "write", "--key", "k", "extra"]),
        (
            "repeated option",
            vec!["test", "write", "--key", "k", "--key", "j"],
        ),
        (
            "enum outside",
            vec!["test", "write", "--key", "k", "--fail", "bogus"],
        ),
        (
            "bad integer",
            vec!["test", "echo", "--text", "hi", "--count", "abc"],
        ),
        (
            "integer above maximum",
            vec!["test", "echo", "--text", "hi", "--count", "101"],
        ),
        (
            "integer below minimum",
            vec!["test", "echo", "--text", "hi", "--count", "-1"],
        ),
        (
            "non-existent date",
            vec!["test", "echo", "--text", "hi", "--due", "2026-02-31"],
        ),
        (
            "ill-shaped date",
            vec!["test", "echo", "--text", "hi", "--due", "31/02"],
        ),
        (
            "string too long",
            vec!["test", "echo", "--text", long.as_str()],
        ),
        ("bare group", vec!["test"]),
        ("bare tool", vec![]),
    ];
    cases
        .into_iter()
        .map(|(c, a)| (c, a.into_iter().map(String::from).collect()))
        .collect()
}

/// `args` + `--store <store>` (when it names an action word) + `--json` if asked.
fn argv(args: &[String], json: bool, store: &Path) -> Vec<String> {
    let mut v = args.to_vec();
    if !v.is_empty() {
        v.push("--store".into());
        v.push(store.display().to_string());
    }
    if json {
        v.push("--json".into());
    }
    v
}

fn spawn(home: &Path, args: &[String]) -> Run {
    arbitre(home, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

#[track_caller]
fn check_usage(r: &Run, json: bool, case: &str) {
    assert_eq!(r.code, 2, "{case}: {}", r.stderr);
    if json {
        let (c, n, _) = failure(&doc(r));
        assert_eq!((c.as_str(), n), ("usage", 2), "{case}");
    } else {
        assert_eq!(r.stdout, "", "{case}");
        assert!(r.stderr.contains("error"), "{case}: {}", r.stderr);
    }
}

#[test]
fn malformed_on_absent_store_leaves_it_absent() {
    for (case, args) in defects() {
        for json in [false, true] {
            let home = tmp("usage-absent");
            let r = spawn(&home, &argv(&args, json, &home.join("s/arbitre.db")));
            check_usage(&r, json, case);
            assert!(
                std::fs::read_dir(&home).unwrap().next().is_none(),
                "{case}: something created under {}",
                home.display()
            );
            let _ = std::fs::remove_dir_all(home);
        }
    }
}

#[test]
fn malformed_on_populated_store_changes_nothing() {
    for (case, args) in defects() {
        for json in [false, true] {
            let home = tmp("usage-populated");
            let store = home.join("arbitre.db");
            populate(&store);
            let before = snapshot(&store);
            let bytes = std::fs::read(&store).unwrap();
            let r = spawn(&home, &argv(&args, json, &store));
            check_usage(&r, json, case);
            assert_eq!(std::fs::read(&store).unwrap(), bytes, "{case}: file bytes");
            assert_eq!(snapshot(&store), before, "{case}: store reads");
            let _ = std::fs::remove_dir_all(home);
        }
    }
}

#[test]
fn empty_store_path_is_usage() {
    // RG-ST03-5.1.
    for json in [false, true] {
        let home = tmp("usage-empty-store");
        let mut args = vec!["test", "noop", "--store", ""];
        if json {
            args.push("--json");
        }
        check_usage(&arbitre(&home, &args), json, "--store \"\"");
        assert!(std::fs::read_dir(&home).unwrap().next().is_none());
    }
}

#[test]
fn unknown_option_is_named() {
    let home = tmp("usage-colour");
    let r = arbitre(&home, &["test", "echo", "--text", "hi", "--colour", "red"]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("--colour"), "{}", r.stderr);
}

#[test]
fn missing_required_shows_the_action_usage() {
    let home = tmp("usage-missing");
    let r = arbitre(&home, &["test", "echo"]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("--text"), "{}", r.stderr);
    assert!(r.stderr.contains("arbitre test echo"), "{}", r.stderr);
}

#[test]
fn enum_outside_lists_accepted_values() {
    let home = tmp("usage-enum");
    let r = arbitre(&home, &["test", "echo", "--text", "hi", "--level", "huge"]);
    assert_eq!(r.code, 2);
    for v in ["low", "medium", "high"] {
        assert!(r.stderr.contains(v), "{}", r.stderr);
    }
}

#[test]
fn bare_group_and_tool_list_sub_commands() {
    let home = tmp("usage-bare");
    let r = arbitre(&home, &["test"]);
    assert_eq!(r.code, 2);
    assert!(
        r.stderr.contains("echo") && r.stderr.contains("write"),
        "{}",
        r.stderr
    );
    let r = arbitre(&home, &[]);
    assert_eq!(r.code, 2);
    assert!(r.stderr.contains("test"), "{}", r.stderr);
}

#[test]
fn unknown_action_is_usage_not_tool_error() {
    let home = tmp("usage-nope");
    let r = arbitre(&home, &["test", "nope", "--json"]);
    assert_eq!(r.code, 2);
    assert_eq!(failure(&doc(&r)).0, "usage");
}

#[test]
fn json_after_terminator_is_not_json_mode() {
    // RG-ST03-2.1: `--json` counts only before `--`.
    let home = tmp("usage-terminator");
    let r = arbitre(&home, &["test", "echo", "--text", "hi", "--", "--json"]);
    assert_eq!(r.code, 2);
    assert_eq!(r.stdout, "");
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_usage() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let home = tmp("usage-utf8");
    let mut c = cmd(&home, &["test", "echo", "--text"]);
    c.arg(OsStr::from_bytes(b"\xff"));
    let r = run(&mut c);
    assert_eq!(r.code, 2, "{}", r.stderr);
    let mut c = cmd(&home, &["test", "echo", "--json", "--text"]);
    c.arg(OsStr::from_bytes(b"\xff"));
    let r = run(&mut c);
    assert_eq!(r.code, 2);
    assert_eq!(failure(&doc(&r)).0, "usage");
    assert!(std::fs::read_dir(&home).unwrap().next().is_none());
}
