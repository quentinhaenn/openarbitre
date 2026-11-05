//! Shared by the acceptance tests: spawn the binary in a clean environment, read stores.
//! ST-04 tooling (frozen after ST-04; extension point: `surfaces/` only): `network`,
//! `surfaces`, `conformance`.
#![allow(dead_code)]

pub mod conformance;
pub mod network;
pub mod surfaces;

use std::ffi::OsString;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

use arbitre_core::{Author, Change, DeviceId, LocalChange, Position, RecordId, Store, Write};
use arbitre_sqlite::{Options, SqliteStore};

/// Fresh empty directory under the system temp dir.
pub fn tmp(tag: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let d = std::env::temp_dir().join(format!(
        "arbitre-st03-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// The binary with `HOME = home`, no `XDG_DATA_HOME`, no `ARBITRE_STORE`.
pub fn cmd(home: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_arbitre"));
    c.args(args)
        .env_remove("XDG_DATA_HOME")
        .env_remove("ARBITRE_STORE")
        .env("HOME", home);
    c
}

/// `args --store <store>` with `vars`; `HOME` is the store's directory.
pub fn on(store: &Path, vars: &[(OsString, OsString)], args: &[&str]) -> Command {
    let s = store.display().to_string();
    let mut v = args.to_vec();
    v.extend(["--store", s.as_str()]);
    let mut c = cmd(store.parent().expect("store in a directory"), &v);
    c.envs(vars.iter().cloned());
    c
}

pub fn run(c: &mut Command) -> Run {
    conformance::run_timed(c).expect("no hang")
}

pub fn arbitre(home: &Path, args: &[&str]) -> Run {
    run(&mut cmd(home, args))
}

/// The single result document on stdout: one line, `\n`-terminated, nothing else.
#[track_caller]
pub fn doc(r: &Run) -> serde_json::Value {
    let body = r
        .stdout
        .strip_suffix('\n')
        .unwrap_or_else(|| panic!("stdout not `\\n`-terminated: {:?}", r.stdout));
    assert!(!body.contains('\n'), "more than one line: {:?}", r.stdout);
    serde_json::from_str(body).unwrap_or_else(|e| panic!("{e}: {:?}", r.stdout))
}

/// `(category, code, id)` of a failure document, after checking its shape (RG-ST03-2.1).
#[track_caller]
pub fn failure(d: &serde_json::Value) -> (String, i64, String) {
    assert_eq!(d["format"], 1);
    assert_eq!(d["ok"], false);
    let e = &d["error"];
    let id = e["id"].as_str().expect("id").to_string();
    assert!(
        id.starts_with(|c: char| c.is_ascii_lowercase())
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
        "id {id:?}"
    );
    assert!(e["message"].is_string());
    (
        e["category"].as_str().expect("category").to_string(),
        e["code"].as_i64().expect("code"),
        id,
    )
}

/// `data` of a success document.
#[track_caller]
pub fn success(d: &serde_json::Value) -> serde_json::Value {
    assert_eq!(d["format"], 1);
    assert_eq!(d["ok"], true);
    assert!(d.get("error").is_none());
    d["data"].clone()
}

/// Some occurrence of `text` starts on a line that also carries `key` (RG-ST03-2.3).
pub fn shown(rendered: &str, key: &str, text: &str) -> bool {
    rendered.match_indices(text).any(|(p, _)| {
        let start = rendered[..p].rfind('\n').map_or(0, |i| i + 1);
        let end = rendered[p..].find('\n').map_or(rendered.len(), |i| p + i);
        rendered[start..end].contains(key)
    })
}

pub fn ready<F: Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("store future pending"),
    }
}

/// A store at `path` holding two writes; handle dropped.
pub fn populate(path: &Path) {
    let s = SqliteStore::open(path, &Options::default()).expect("open");
    let lc = |id: &str| LocalChange {
        record: RecordId {
            module: "seed".into(),
            category: "item".into(),
            id: id.into(),
        },
        change: Change::field("v", id.as_bytes()),
        author: Author::System,
    };
    ready(s.apply_local(1_000, &[lc("a"), lc("b")])).expect("seed");
}

/// What the store holds: every write since `START` and the device id.
pub fn snapshot(path: &Path) -> (Vec<(Position, Write)>, DeviceId) {
    let s = SqliteStore::open(path, &Options::default()).expect("open");
    let w = ready(s.writes_since(Position::START)).expect("writes");
    let d = ready(s.device()).expect("device");
    (w, d)
}
