//! Human/JSON equivalence, input kinds, help and version (ST-03 E-ST03-2, E-ST03-10;
//! matrices US-2, US-4; RG-ST03-1.4, 2.1 to 2.4, 4.3, 4.7).

mod support;

use std::path::Path;

use serde_json::json;
use support::*;

/// Echoed data without its `remark`, after checking the remark is a string.
#[track_caller]
fn echoed(d: &serde_json::Value) -> serde_json::Value {
    let mut data = success(d);
    let o = data.as_object_mut().expect("object");
    assert!(o.remove("remark").expect("remark").is_string());
    data
}

#[test]
fn nominal_json() {
    // Matrix US-2 rows 1, 3.
    let home = tmp("out-nominal");
    let r = on_home(&home, &["test", "echo", "--text", "hi"], true);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(
        echoed(&doc(&r)),
        json!({"flag": false, "tag": [], "text": "hi"})
    );
}

#[test]
fn every_input_kind() {
    // Matrix US-4 rows 1, 3.
    let home = tmp("out-kinds");
    let r = on_home(
        &home,
        &[
            "test",
            "echo",
            "--text",
            "hi",
            "--count",
            "3",
            "--flag",
            "--level",
            "high",
            "--due",
            "2024-02-29",
            "--tag",
            "a",
            "--tag",
            "b",
            "--tag",
            "a",
            "--note",
            "n",
        ],
        true,
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(
        echoed(&doc(&r)),
        json!({
            "count": 3, "due": "2024-02-29", "flag": true, "level": "high",
            "note": "n", "tag": ["a", "b", "a"], "text": "hi"
        })
    );
}

#[test]
fn optional_absent_is_an_absent_key() {
    // Matrix US-4 row 2.
    let home = tmp("out-optional");
    let d = echoed(&doc(&on_home(
        &home,
        &["test", "echo", "--text", "hi"],
        true,
    )));
    for k in ["note", "count", "level", "due"] {
        assert!(d.get(k).is_none(), "{k}");
    }
}

#[test]
fn global_options_anywhere() {
    let home = tmp("out-global");
    let s = store(&home).display().to_string();
    let r = arbitre(
        &home,
        &[
            "--json",
            "--store",
            s.as_str(),
            "test",
            "echo",
            "--text",
            "hi",
        ],
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(echoed(&doc(&r))["text"], "hi");
}

#[test]
fn human_text_shows_every_leaf() {
    // Matrix US-2 row 2, US-3 row 11.
    let home = tmp("out-human");
    let args = [
        "test",
        "echo",
        "--text",
        "hi",
        "--count",
        "7",
        "--flag",
        "--level",
        "low",
        "--due",
        "2026-11-04",
        "--tag",
        "red",
        "--tag",
        "blue",
        "--note",
        "nn",
    ];
    let d = success(&doc(&on_home(&home, &args, true)));
    let r = on_home(&home, &args, false);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for (k, v) in d.as_object().unwrap() {
        let texts: Vec<String> = match v {
            serde_json::Value::Array(a) => {
                a.iter().map(|x| x.as_str().unwrap().to_string()).collect()
            }
            serde_json::Value::String(x) => vec![x.clone()],
            other => vec![other.to_string()],
        };
        for t in texts {
            assert!(shown(&r.stdout, k, &t), "{k}: {t:?} not in {:?}", r.stdout);
        }
    }
}

#[test]
fn free_text_escaped_in_json_verbatim_in_human() {
    // Matrix US-2 row 4, RG-ST03-2.4.
    let home = tmp("out-free");
    let x = "a\nb \"q\" \\ \u{1} é 日本 /";
    let r = on_home(&home, &["test", "echo", "--text", x], true);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout
            .contains(r#""text":"a\nb \"q\" \\ \u0001 é 日本 /""#),
        "{}",
        r.stdout
    );
    assert_eq!(echoed(&doc(&r))["text"], x);
    let r = on_home(&home, &["test", "echo", "--text", x], false);
    assert!(r.stdout.contains(x), "{:?}", r.stdout);
}

#[test]
fn pure_command() {
    // Matrix US-2 row 5.
    let home = tmp("out-noop");
    let r = on_home(&home, &["test", "noop"], true);
    assert_eq!(
        (r.code, r.stdout.as_str()),
        (0, "{\"data\":{},\"format\":1,\"ok\":true}\n")
    );
    let r = on_home(&home, &["test", "noop"], false);
    assert_eq!((r.code, r.stdout.as_str()), (0, ""));
}

#[test]
fn failure_details() {
    // Matrix US-2 row 6, Q-DA1 (a).
    let home = tmp("out-refuse");
    let r = on_home(&home, &["test", "refuse"], true);
    assert_eq!(r.code, 1);
    let d = doc(&r);
    assert_eq!(failure(&d).0, "refusal");
    assert_eq!(d["data"], json!({"item": "x"}));
    let r = on_home(&home, &["test", "refuse"], false);
    assert_eq!((r.code, r.stdout.as_str()), (1, ""));
    assert!(
        r.stderr.lines().any(|l| l.starts_with("error: ")),
        "{}",
        r.stderr
    );
    assert!(shown(&r.stderr, "item", "x"), "{}", r.stderr);
    let d = doc(&on_home(&home, &["test", "fail"], true));
    assert_eq!(failure(&d).0, "tool_error");
    assert!(d.get("data").is_none(), "{d}");
}

#[test]
fn diagnostics_stay_on_stderr() {
    // Matrix US-2 row 7.
    let home = tmp("out-noisy");
    let r = on_home(&home, &["test", "noisy"], true);
    assert_eq!(r.code, 0);
    assert_eq!(success(&doc(&r)), json!({}));
    assert!(!r.stderr.is_empty());
}

#[test]
fn same_value_same_bytes() {
    // Matrix US-2 row 8.
    let home = tmp("out-bytes");
    let a = on_home(
        &home,
        &["test", "echo", "--text", "hi", "--count", "3", "--tag", "t"],
        true,
    );
    let b = on_home(
        &home,
        &["test", "echo", "--tag", "t", "--count", "3", "--text", "hi"],
        true,
    );
    let c = on_home(
        &home,
        &["test", "echo", "--text", "hi", "--count", "3", "--tag", "t"],
        true,
    );
    assert_eq!(a.code, 0);
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(a.stdout, c.stdout);
}

#[test]
fn action_and_group_both_run() {
    // Matrix US-4 row 7, Q-B (a).
    let home = tmp("out-group");
    for args in [&["test", "group"][..], &["test", "group", "child"]] {
        let r = on_home(&home, args, true);
        assert_eq!(r.code, 0, "{args:?}: {}", r.stderr);
        assert_eq!(success(&doc(&r)), json!({}));
    }
}

#[test]
fn foundation_path() {
    // Matrix US-4 row 9.
    let home = tmp("out-foundation");
    let r = on_home(&home, &["test-foundation"], true);
    assert_eq!(r.code, 0, "{}", r.stderr);
}

// --- E-ST03-10: help and version (RG-ST03-1.3, 1.4) ---

#[track_caller]
fn nothing_created(home: &Path) {
    assert!(
        std::fs::read_dir(home).unwrap().next().is_none(),
        "store touched"
    );
}

#[test]
fn help_and_version_succeed_without_store() {
    let cases: [&[&str]; 6] = [
        &["--help"],
        &["-h"],
        &["test", "echo", "--help"],
        &["test", "--help"],
        &["--version"],
        &["-V"],
    ];
    for args in cases {
        let home = tmp("out-help");
        let r = arbitre(&home, args);
        assert_eq!(r.code, 0, "{args:?}: {}", r.stderr);
        assert!(!r.stdout.is_empty(), "{args:?}");
        let mut j = args.to_vec();
        j.push("--json");
        let r = arbitre(&home, &j);
        assert_eq!(r.code, 0, "{j:?}: {}", r.stderr);
        let data = success(&doc(&r));
        let key = if args[args.len() - 1].starts_with("--v") || args[args.len() - 1] == "-V" {
            "version"
        } else {
            "help"
        };
        assert!(data[key].is_string(), "{j:?}: {data}");
        assert_eq!(data.as_object().unwrap().len(), 1);
        nothing_created(&home);
    }
}

#[test]
fn version_text() {
    let home = tmp("out-version");
    let r = arbitre(&home, &["--version"]);
    assert!(r.stdout.contains(env!("CARGO_PKG_VERSION")), "{}", r.stdout);
}

#[test]
fn action_help_lists_every_option() {
    // RG-ST03-4.7: options, value names, enum values, summary, from the descriptor only.
    let home = tmp("out-help-echo");
    let r = arbitre(&home, &["test", "echo", "--help"]);
    for s in [
        "Echoes its input with a remark.",
        "--text",
        "--count",
        "--flag",
        "--level",
        "--due",
        "--tag",
        "--note",
        "--json",
        "--store",
        "<TEXT>",
        "<INTEGER>",
        "<DATE>",
        "low",
        "medium",
        "high",
        "A due date.",
    ] {
        assert!(r.stdout.contains(s), "{s:?} missing from {}", r.stdout);
    }
}

#[test]
fn sub_commands_listed_in_name_order() {
    let home = tmp("out-help-order");
    let r = arbitre(&home, &["test", "--help"]);
    let pos = |w: &str| {
        r.stdout
            .lines()
            .position(|l| l.trim_start().starts_with(w))
            .unwrap_or_else(|| panic!("{w} missing from {}", r.stdout))
    };
    let names = [
        "bad-output",
        "echo",
        "fail",
        "group",
        "noisy",
        "noop",
        "panic",
        "read",
        "refuse",
        "signal",
        "write",
    ];
    let ps: Vec<usize> = names.iter().map(|n| pos(n)).collect();
    assert!(ps.windows(2).all(|w| w[0] < w[1]), "{}", r.stdout);
}
