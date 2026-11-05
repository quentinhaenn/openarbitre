//! Store location (ST-03 E-ST03-5, matrix US-5; RG-ST03-5.1 to 5.3, PROP-ST03-2).

mod support;

use std::path::{Path, PathBuf};

use support::*;

/// Platform default under `home` (RG-ST03-5.1); this test runs on Linux and macOS only.
fn default_under(home: &Path) -> PathBuf {
    if cfg!(target_os = "macos") {
        home.join("Library/Application Support/arbitre/arbitre.db")
    } else {
        home.join(".local/share/arbitre/arbitre.db")
    }
}

#[test]
fn default_location() {
    // Row 1.
    let home = tmp("loc-default");
    let r = arbitre(&home, &["test", "write", "--key", "k"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(default_under(&home).exists());
}

#[test]
fn empty_env_falls_back_to_default() {
    let home = tmp("loc-empty-env");
    let r = run(cmd(&home, &["test", "write", "--key", "k"]).env("ARBITRE_STORE", ""));
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(default_under(&home).exists());
}

#[test]
fn env_location() {
    // Row 2.
    let home = tmp("loc-env");
    let a = home.join("a.db");
    let r = run(cmd(&home, &["test", "write", "--key", "k"]).env("ARBITRE_STORE", &a));
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(a.exists());
    assert!(!default_under(&home).exists());
}

#[test]
fn option_beats_env() {
    // Row 3.
    let home = tmp("loc-option");
    let (a, b) = (home.join("a.db"), home.join("b.db"));
    let b_text = b.display().to_string();
    let r = run(cmd(
        &home,
        &["test", "write", "--key", "k", "--store", b_text.as_str()],
    )
    .env("ARBITRE_STORE", &a));
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(b.exists());
    assert!(!a.exists());
}

#[test]
fn relative_path_from_current_directory() {
    let home = tmp("loc-relative");
    let r =
        run(cmd(&home, &["test", "write", "--key", "k", "--store", "rel.db"]).current_dir(&home));
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(home.join("rel.db").exists());
}

#[cfg(unix)]
#[test]
fn uncreatable_directory() {
    // Row 4: read-only parent.
    use std::os::unix::fs::PermissionsExt;
    let home = tmp("loc-readonly");
    let ro = home.join("ro");
    std::fs::create_dir(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
    if std::fs::create_dir(ro.join("probe")).is_ok() {
        eprintln!("running as root: permissions not enforced, case skipped");
        return;
    }
    let target = ro.join("x/a.db").display().to_string();
    let r = arbitre(
        &home,
        &["test", "read", "--store", target.as_str(), "--json"],
    );
    let created = ro.join("x").exists();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(r.code, 3, "{}", r.stderr);
    assert_eq!(failure(&doc(&r)).2, "storage_unavailable");
    assert!(!created);
}

#[test]
fn no_home() {
    // Row 5.
    let home = tmp("loc-no-home");
    let r = run(cmd(&home, &["test", "read", "--json"]).env_remove("HOME"));
    assert_eq!(r.code, 3, "{}", r.stderr);
    assert_eq!(failure(&doc(&r)).2, "no_default_store");
}

#[test]
fn same_data_across_processes() {
    // Row 6.
    let home = tmp("loc-shared");
    let a = home.join("shared.db");
    let r = run(cmd(&home, &["test", "write", "--key", "k1"]).env("ARBITRE_STORE", &a));
    assert_eq!(r.code, 0, "{}", r.stderr);
    let r = run(cmd(&home, &["test", "read", "--json"]).env("ARBITRE_STORE", &a));
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(success(&doc(&r))["keys"], serde_json::json!(["k1"]));
}
