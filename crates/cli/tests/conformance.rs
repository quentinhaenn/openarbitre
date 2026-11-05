//! Generic conformance test: every toggleable module of the build, every registered surface
//! (ST-04 E-ST04-5; specs RG-ST04-4.2, 4.3, 5.1 to 5.3; plan P-15, P-16). Frozen: a new
//! module or surface joins without editing this file.

mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Value as Json, json};
use support::conformance::{
    ModuleInfo, last_position, minimal_input, modules, modules_written_after, run_timed,
};
use support::network::{Mode, Observer};
use support::surfaces::{self, Env};
use support::*;

const NOW: &str = "1768485900000";

struct Case {
    home: PathBuf,
    env: Env,
}

/// Fresh store, every toggleable module enabled, observer variables applied.
fn fresh(tag: &str, observer: &Observer) -> Case {
    let home = tmp(tag);
    let env = Env {
        store: home.join("arbitre.db"),
        vars: observer.env(),
    };
    let c = Case { home, env };
    for m in toggleable(&c) {
        c.switch(&m.id, true);
    }
    c
}

fn toggleable(c: &Case) -> Vec<ModuleInfo> {
    modules(&c.home, &c.env.store)
}

impl Case {
    fn cmd(&self, args: &[&str]) -> std::process::Command {
        let s = self.env.store.display().to_string();
        let mut v = args.to_vec();
        v.extend(["--store", s.as_str(), "--json"]);
        let mut c = cmd(&self.home, &v);
        c.envs(self.env.vars.iter().cloned())
            .env("ARBITRE_TEST_NOW", NOW);
        c
    }

    fn switch(&self, m: &str, on: bool) {
        let verb = if on { "enable" } else { "disable" };
        let r = run_timed(&mut self.cmd(&["module", verb, "--module", m])).expect("no hang");
        assert_eq!(r.code, 0, "{verb} {m}: {}", r.stdout);
    }

    /// Copy of the store file into a new case (same device, same data).
    fn copy(&self, tag: &str) -> Case {
        let home = tmp(tag);
        let store = home.join("arbitre.db");
        std::fs::copy(&self.env.store, &store).expect("copy store");
        Case {
            home,
            env: Env {
                store,
                vars: self.env.vars.clone(),
            },
        }
    }
}

fn names(m: &ModuleInfo) -> BTreeSet<String> {
    m.actions
        .iter()
        .map(|a| a["name"].as_str().expect("name").to_string())
        .collect()
}

fn input_of(a: &Json) -> Json {
    let schema: Json = serde_json::from_str(a["input"].as_str().expect("input schema text"))
        .expect("input schema is JSON");
    minimal_input(&schema)
}

fn path(name: &str) -> Vec<&str> {
    name.split('.').collect()
}

/// Options of a minimal input (the RG-ST03-4.3 table; lists are empty, booleans false).
fn options(input: &Json) -> Vec<String> {
    let mut v = Vec::new();
    for (k, x) in input.as_object().expect("object") {
        let opt = format!("--{}", k.replace('_', "-"));
        match x {
            Json::String(s) => v.extend([opt, s.clone()]),
            Json::Number(n) => v.extend([opt, n.to_string()]),
            _ => {}
        }
    }
    v
}

fn call_cli(c: &Case, a: &Json) -> Option<Run> {
    let name = a["name"].as_str().expect("name");
    let opts = options(&input_of(a));
    let mut args = path(name);
    args.extend(opts.iter().map(String::as_str));
    run_timed(&mut c.cmd(&args))
}

fn surface_names(c: &Case) -> Vec<(String, BTreeSet<String>)> {
    surfaces::all()
        .iter()
        .map(|s| (s.name().to_string(), s.actions(&c.env)))
        .collect()
}

#[test]
fn checks_1_2_visible_when_enabled_hidden_and_refused_when_disabled() {
    let o = Observer::start(Mode::Cut);
    let c = fresh("conf-12", &o);
    for m in toggleable(&c) {
        let own = names(&m);
        for (s, seen) in surface_names(&c) {
            assert!(own.is_subset(&seen), "{s}: {} visible", m.id);
        }
        c.switch(&m.id, false);
        for (s, seen) in surface_names(&c) {
            assert!(own.is_disjoint(&seen), "{s}: {} hidden", m.id);
        }
        let before = snapshot(&c.env.store);
        for a in &m.actions {
            let r = call_cli(&c, a).expect("no hang");
            assert_eq!(r.code, 4, "{}", a["name"]);
        }
        assert_eq!(snapshot(&c.env.store), before, "{}: nothing written", m.id);
        c.switch(&m.id, true);
    }
}

#[test]
fn checks_3_4_data_kept_and_namespace_isolated() {
    let o = Observer::start(Mode::Cut);
    let c = fresh("conf-34", &o);
    for m in toggleable(&c) {
        let after_setup = last_position(&c.env.store);
        for a in m.actions.iter().filter(|a| a["mutating"] == true) {
            call_cli(&c, a).expect("no hang");
        }
        for w in modules_written_after(&c.env.store, after_setup) {
            assert_eq!(w, m.id, "{} wrote outside its namespace", m.id);
        }
        let ns = |s: &Path| -> Vec<_> {
            snapshot(s)
                .0
                .into_iter()
                .filter(|(_, w)| w.record.module == m.id)
                .collect()
        };
        let before = ns(&c.env.store);
        c.switch(&m.id, false);
        c.switch(&m.id, true);
        assert_eq!(ns(&c.env.store), before, "{}: data kept", m.id);
    }
}

#[test]
fn check_5_every_subset_works_alone() {
    let o = Observer::start(Mode::Cut);
    let base = fresh("conf-5", &o);
    let all = toggleable(&base);
    let ids: Vec<_> = all.iter().map(|m| m.id.clone()).collect();
    let pairs_only = ids.len() > 6;
    for mask in 1u32..(1 << ids.len().min(16)) {
        let subset: Vec<_> = (0..ids.len()).filter(|i| mask & (1 << i) != 0).collect();
        if pairs_only && subset.len() > 2 {
            continue;
        }
        for m in subset.iter().map(|&i| &all[i]) {
            for a in &m.actions {
                let full = base.copy("conf-5-full");
                let part = base.copy("conf-5-part");
                for (i, id) in ids.iter().enumerate() {
                    if !subset.contains(&i) {
                        part.switch(id, false);
                    }
                }
                let (x, y) = (call_cli(&full, a), call_cli(&part, a));
                let (x, y) = (x.expect("no hang"), y.expect("no hang"));
                assert_eq!((x.code, &x.stdout), (y.code, &y.stdout), "{}", a["name"]);
            }
        }
    }
}

#[test]
fn check_6_network_cut() {
    let o = Observer::start(Mode::Cut);
    let c = fresh("conf-6", &o);
    o.take();
    for m in toggleable(&c) {
        for a in &m.actions {
            let r = call_cli(&c, a).unwrap_or_else(|| panic!("{} hangs", a["name"]));
            let declared = !a["requires_network"].as_array().expect("list").is_empty();
            if declared {
                assert!([0, 1, 3].contains(&r.code), "{}: {}", a["name"], r.stdout);
                if r.code != 0 {
                    let (_, _, id) = failure(&doc(&r));
                    assert!(!id.is_empty());
                }
            } else {
                assert!([0, 1].contains(&r.code), "{}: {}", a["name"], r.stdout);
                assert!(o.take().is_empty(), "{} tried the network", a["name"]);
            }
        }
    }
}

#[test]
fn check_7_parity_across_surfaces() {
    let o = Observer::start(Mode::Cut);
    let base = fresh("conf-7", &o);
    let catalogue: BTreeSet<String> = toggleable(&base).iter().flat_map(names).collect();
    for (s, seen) in surface_names(&base) {
        let modules_seen: BTreeSet<_> = seen
            .into_iter()
            .filter(|n| !n.starts_with("arbitre."))
            .collect();
        assert_eq!(modules_seen, catalogue, "{s}");
    }
    for m in toggleable(&base) {
        for a in &m.actions {
            let name = a["name"].as_str().expect("name");
            let calls: Vec<_> = surfaces::all()
                .iter()
                .map(|s| s.call(&base.copy("conf-7-call").env, name, &input_of(a)))
                .collect();
            for w in calls.windows(2) {
                assert_eq!((w[0].code, &w[0].doc), (w[1].code, &w[1].doc), "{name}");
            }
        }
    }
}

#[test]
fn fake_crud_scenario_offline() {
    // RG-ST04-4.2: the full scenario succeeds with the network cut.
    let o = Observer::start(Mode::Cut);
    let c = fresh("conf-crud", &o);
    o.take();
    for args in [
        &["fake-a", "put", "--key", "k", "--value", "1"][..],
        &["fake-a", "get", "--key", "k"],
        &["fake-a", "list"],
        &["fake-a", "update", "--key", "k", "--value", "2"],
        &["fake-a", "delete", "--key", "k"],
    ] {
        let r = run_timed(&mut c.cmd(args)).expect("no hang");
        assert_eq!(r.code, 0, "{args:?}: {}", r.stdout);
    }
    let r = run_timed(&mut c.cmd(&["fake-a", "get", "--key", "k"])).expect("no hang");
    assert_eq!(r.code, 1, "deleted ⇒ refusal not_found");
    assert!(o.take().is_empty());
}

#[test]
fn minimal_input_of_each_subset_type() {
    // RG-ST04-5.3, INV-ST04-7.
    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["s", "i", "e", "d", "b", "l"],
        "properties": {
            "s": {"type": "string", "minLength": 3},
            "i": {"type": "integer", "minimum": 5},
            "j": {"type": "integer"},
            "e": {"type": "string", "enum": ["low", "high"]},
            "d": {"type": "string", "format": "date"},
            "b": {"type": "boolean"},
            "l": {"type": "array", "items": {"type": "string"}},
            "o": {"type": "string"}
        }
    });
    assert_eq!(
        minimal_input(&schema),
        json!({"s": "aaa", "i": 5, "e": "low", "d": "2000-01-01", "b": false, "l": []})
    );
    let empty = json!({"type": "object", "properties": {}, "additionalProperties": false});
    assert_eq!(minimal_input(&empty), json!({}));
}

#[test]
fn a_module_with_no_action_passes_the_loops() {
    // Matrix US-5 row 4, at helper level.
    let m = ModuleInfo {
        id: "empty".into(),
        enabled: true,
        actions: vec![],
    };
    assert!(names(&m).is_empty());
    assert!(m.actions.iter().all(|a| !input_of(a).is_null()));
}
