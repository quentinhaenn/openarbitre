//! Store location, opening, storage error mapping, single-poll executor
//! (ST-03 specs RG-ST03-3.3, 5.1 to 5.3; plan P-6, P-13).

use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::task::{Context, Poll, Waker};

use arbitre_core::{ActionError, ApplyError, Cause, StoreError};
use arbitre_sqlite::{Options, SqliteStore};

/// Store file path: `flag` > non-empty `ARBITRE_STORE` > platform default (RG-ST03-5.1).
/// `env` reads a variable; `os` is `std::env::consts::OS` (no `cfg` here, plan P-13).
/// Linux and other Unix: absolute `XDG_DATA_HOME`, else `HOME/.local/share`; macOS:
/// `HOME/Library/Application Support`; `windows` or no usable `HOME` → `no_default_store`.
pub fn location(
    flag: Option<&OsStr>,
    env: impl Fn(&str) -> Option<OsString>,
    os: &str,
) -> Result<PathBuf, ActionError> {
    if let Some(f) = flag {
        return Ok(PathBuf::from(f));
    }
    let set = |k: &str| env(k).filter(|v| !v.is_empty());
    if let Some(p) = set("ARBITRE_STORE") {
        return Ok(PathBuf::from(p));
    }
    let no_default = || {
        ActionError::tool_error(
            "no_default_store",
            "no store location: pass --store or set ARBITRE_STORE",
        )
    };
    let home = set("HOME").map(PathBuf::from);
    let base = match os {
        "windows" => return Err(no_default()),
        "macos" => home
            .ok_or_else(no_default)?
            .join("Library/Application Support"),
        _ => match set("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
        {
            Some(x) => x,
            None => home.ok_or_else(no_default)?.join(".local/share"),
        },
    };
    Ok(base.join("arbitre").join("arbitre.db"))
}

/// Creates missing parent directories, then opens with default options (plan R-4).
/// Failures map through `map_store`; an uncreatable directory → `storage_unavailable`.
pub fn open(path: &Path) -> Result<SqliteStore, ActionError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| {
            ActionError::tool_error(
                "storage_unavailable",
                format!("cannot create {}: {e}", dir.display()),
            )
        })?;
    }
    SqliteStore::open(path, &Options::default()).map_err(map_store)
}

/// `StoreError` → `ToolError` with the ids of RG-ST03-3.3.
pub fn map_store(e: StoreError) -> ActionError {
    let (id, msg) = match e {
        StoreError::StorageFull => ("storage_full", "storage is full".to_string()),
        StoreError::Corrupted(d) => ("storage_corrupted", format!("store damaged: {d:?}")),
        StoreError::Unavailable(Cause::Busy) => ("storage_busy", "store is busy".to_string()),
        StoreError::Unavailable(c @ Cause::IncompatibleVersion { .. }) => {
            ("storage_incompatible_version", format!("{c:?}"))
        }
        StoreError::Unavailable(c) => ("storage_unavailable", format!("store unavailable: {c:?}")),
    };
    ActionError::tool_error(id, msg)
}

/// `ApplyError` → `ToolError`: `Rejected` → `write_rejected`, `Store` → `map_store`.
pub fn map_apply(e: ApplyError) -> ActionError {
    match e {
        ApplyError::Rejected(r) => {
            ActionError::tool_error("write_rejected", format!("write rejected: {r:?}"))
        }
        ApplyError::Store(s) => map_store(s),
    }
}

/// One poll on a no-op waker (ST-02 futures are ready, spike S1-A); `Pending` → `internal_error`.
pub fn block_on<F: Future>(fut: F) -> Result<F::Output, ActionError> {
    match std::pin::pin!(fut).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => Ok(v),
        Poll::Pending => Err(ActionError::tool_error(
            "internal_error",
            "store future not ready",
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use arbitre_core::{Category, Cause, Damage, Rejection};

    use super::*;

    fn env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
        let m: BTreeMap<String, OsString> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), OsString::from(v)))
            .collect();
        move |k| m.get(k).cloned()
    }

    fn at(os: &str, vars: &[(&str, &str)]) -> Result<PathBuf, ActionError> {
        location(None, env(vars), os)
    }

    #[track_caller]
    fn tool(e: ActionError, id: &str) {
        assert_eq!(e.category(), Category::ToolError);
        assert_eq!(e.id(), id);
    }

    fn tmp() -> PathBuf {
        static N: AtomicUsize = AtomicUsize::new(0);
        let d = std::env::temp_dir().join(format!(
            "arbitre-cli-store-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn flag_beats_env_beats_default() {
        let vars = [("ARBITRE_STORE", "/e/a.db"), ("HOME", "/h")];
        assert_eq!(
            location(Some(OsStr::new("b.db")), env(&vars), "linux").unwrap(),
            PathBuf::from("b.db")
        );
        assert_eq!(at("linux", &vars).unwrap(), PathBuf::from("/e/a.db"));
        assert_eq!(at("macos", &vars).unwrap(), PathBuf::from("/e/a.db"));
        assert_eq!(at("windows", &vars).unwrap(), PathBuf::from("/e/a.db"));
    }

    #[test]
    fn empty_env_ignored() {
        assert_eq!(
            at("linux", &[("ARBITRE_STORE", ""), ("HOME", "/h")]).unwrap(),
            PathBuf::from("/h/.local/share/arbitre/arbitre.db")
        );
    }

    #[test]
    fn linux_defaults() {
        assert_eq!(
            at("linux", &[("XDG_DATA_HOME", "/x"), ("HOME", "/h")]).unwrap(),
            PathBuf::from("/x/arbitre/arbitre.db")
        );
        assert_eq!(
            at("linux", &[("XDG_DATA_HOME", "rel"), ("HOME", "/h")]).unwrap(),
            PathBuf::from("/h/.local/share/arbitre/arbitre.db")
        );
        assert_eq!(
            at("linux", &[("XDG_DATA_HOME", ""), ("HOME", "/h")]).unwrap(),
            PathBuf::from("/h/.local/share/arbitre/arbitre.db")
        );
        assert_eq!(
            at("linux", &[("HOME", "/h")]).unwrap(),
            PathBuf::from("/h/.local/share/arbitre/arbitre.db")
        );
        assert_eq!(
            at("freebsd", &[("HOME", "/h")]).unwrap(),
            PathBuf::from("/h/.local/share/arbitre/arbitre.db")
        );
    }

    #[test]
    fn macos_default() {
        assert_eq!(
            at("macos", &[("XDG_DATA_HOME", "/x"), ("HOME", "/h")]).unwrap(),
            PathBuf::from("/h/Library/Application Support/arbitre/arbitre.db")
        );
    }

    #[test]
    fn no_default_store() {
        tool(at("linux", &[]).unwrap_err(), "no_default_store");
        tool(at("macos", &[]).unwrap_err(), "no_default_store");
        tool(
            at("linux", &[("HOME", "")]).unwrap_err(),
            "no_default_store",
        );
        tool(
            at("windows", &[("HOME", "/h")]).unwrap_err(),
            "no_default_store",
        );
    }

    #[test]
    fn store_errors_mapped() {
        let cases = [
            (StoreError::StorageFull, "storage_full"),
            (StoreError::Corrupted(Damage::Layout), "storage_corrupted"),
            (
                StoreError::Corrupted(Damage::NotAStore),
                "storage_corrupted",
            ),
            (StoreError::Unavailable(Cause::Busy), "storage_busy"),
            (
                StoreError::Unavailable(Cause::IncompatibleVersion {
                    stored: 2,
                    supported: 1,
                }),
                "storage_incompatible_version",
            ),
            (
                StoreError::Unavailable(Cause::Io("disk".into())),
                "storage_unavailable",
            ),
            (
                StoreError::Unavailable(Cause::RandomSource),
                "storage_unavailable",
            ),
        ];
        for (e, id) in cases {
            tool(map_store(e.clone()), id);
            tool(map_apply(ApplyError::Store(e)), id);
        }
        tool(
            map_apply(ApplyError::Rejected(Rejection::LocalIdentityReused {
                index: 0,
            })),
            "write_rejected",
        );
    }

    #[test]
    fn open_creates_parent_directories() {
        let d = tmp();
        let p = d.join("a/b/arbitre.db");
        drop(open(&p).expect("opened"));
        assert!(p.exists());
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn uncreatable_directory_is_unavailable() {
        let d = tmp();
        let file = d.join("file");
        std::fs::write(&file, b"x").unwrap();
        tool(
            open(&file.join("x/arbitre.db")).err().expect("refused"),
            "storage_unavailable",
        );
        let _ = std::fs::remove_dir_all(d);
    }

    #[test]
    fn block_on_ready_and_pending() {
        assert_eq!(block_on(async { 7 }).unwrap(), 7);
        tool(
            block_on(std::future::pending::<()>()).unwrap_err(),
            "internal_error",
        );
    }
}
