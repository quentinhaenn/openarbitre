//! Helpers of the generic conformance test (ST-04 specs RG-ST04-5.1 to 5.3; plan P-15, P-16).

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value as Json, json};

use super::{Run, doc, on, snapshot, success};

/// Longest a command may run before the check fails as a hang (RG-ST04-4.2, 4.3).
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// One module as `arbitre module list --json` gives it.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: String,
    /// Descriptor objects (`name`, `summary`, `input`, `output`, `mutating`, `logged_as`,
    /// `requires_network`); empty when disabled.
    pub actions: Vec<serde_json::Value>,
}

/// Modules of the build, read from the binary on `store` (RG-ST04-5.1).
pub fn modules(store: &Path) -> Vec<ModuleInfo> {
    let r = run_timed(&mut on(store, &[], &["module", "list", "--json"])).expect("no hang");
    success(&doc(&r))["modules"]
        .as_array()
        .expect("modules")
        .iter()
        .map(|m| ModuleInfo {
            id: m["id"].as_str().expect("id").to_string(),
            actions: m["actions"].as_array().expect("actions").clone(),
        })
        .collect()
}

/// Minimal conforming input of an input JSON Schema (RG-ST04-5.3).
pub fn minimal_input(schema: &serde_json::Value) -> serde_json::Value {
    let required = schema["required"].as_array().map_or(&[][..], Vec::as_slice);
    required
        .iter()
        .map(|k| {
            let k = k.as_str().expect("property name");
            (k.to_string(), minimal_value(&schema["properties"][k]))
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}

fn minimal_value(p: &Json) -> Json {
    if let Some(first) = p["enum"].get(0) {
        return first.clone();
    }
    match (p["type"].as_str(), p["format"].as_str()) {
        (Some("string"), Some("date")) => json!("2000-01-01"),
        (Some("string"), _) => {
            json!("a".repeat(p["minLength"].as_u64().unwrap_or(1).max(1) as usize))
        }
        (Some("integer"), _) => json!(p["minimum"].as_i64().unwrap_or(0)),
        (Some("boolean"), _) => json!(false),
        (Some("array"), _) => json!([]),
        _ => panic!("outside the RG-ST03-4.3 subset: {p}"),
    }
}

/// Subsets of `n` modules checked by check 5 (RG-ST04-5.2 item 5): all non-empty ones up to
/// 6 modules, singletons and pairs above.
pub fn subsets(n: usize) -> Vec<Vec<usize>> {
    if n <= 6 {
        return (1..1u32 << n)
            .map(|mask| (0..n).filter(|i| mask & (1 << i) != 0).collect())
            .collect();
    }
    (0..n)
        .flat_map(|i| (i..n).map(move |j| if i == j { vec![i] } else { vec![i, j] }))
        .collect()
}

/// Runs `c`; `None` if it outlives `TIMEOUT` (then killed and reaped, DA F-12).
pub fn run_timed(c: &mut Command) -> Option<Run> {
    let mut child = c
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn arbitre");
    // Pipes drained on threads: a full pipe would block the child and fake a hang.
    let (out, err) = (drain(child.stdout.take()), drain(child.stderr.take()));
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().expect("wait") {
            break s;
        }
        if start.elapsed() > TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        thread::sleep(Duration::from_millis(5));
    };
    Some(Run {
        code: status.code().expect("exit code, not a signal"),
        stdout: String::from_utf8(out.join().expect("stdout")).expect("UTF-8 stdout"),
        stderr: String::from_utf8_lossy(&err.join().expect("stderr")).into_owned(),
    })
}

fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<Vec<u8>> {
    let mut pipe = pipe.expect("piped");
    thread::spawn(move || {
        let mut v = Vec::new();
        let _ = pipe.read_to_end(&mut v);
        v
    })
}

/// Position of the last write in `store` (0 if none): check 4 compares what comes after.
pub fn last_position(store: &Path) -> u64 {
    snapshot(store).0.last().map_or(0, |(p, _)| p.0)
}

/// Modules of the writes after `after` in `store`.
pub fn modules_written_after(store: &Path, after: u64) -> Vec<String> {
    snapshot(store)
        .0
        .into_iter()
        .filter(|(p, _)| p.0 > after)
        .map(|(_, w)| w.record.module)
        .collect()
}
