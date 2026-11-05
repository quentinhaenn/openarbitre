//! Generic conformance test: every toggleable module of the build, every registered surface
//! (ST-04 E-ST04-5; specs RG-ST04-4.2, 4.3, 5.1 to 5.3; plan P-15, P-16). Frozen: a new
//! module or surface joins without editing this file.

mod support;

use std::collections::BTreeSet;

use serde_json::{Value as Json, json};
use support::conformance::{
    ModuleInfo, last_position, minimal_input, modules, modules_written_after, run_timed, subsets,
};
use support::network::{Mode, Observer};
use support::surfaces::cli::Cli;
use support::surfaces::{self, Called, Env, Surface};
use support::*;

const NOW: &str = "1768485900000";

/// Hosts each network kind may reach in open mode (RG-ST04-4.4): `(kind, host)`. Empty in
/// ST-04; an adapter story adds its rows.
const ALLOWED: &[(&str, &str)] = &[];

/// Fresh store, every toggleable module enabled, observer variables and pinned time applied.
fn fresh(tag: &str, observer: &Observer) -> Env {
    let mut vars = observer.env();
    vars.push(("ARBITRE_TEST_NOW".into(), NOW.into()));
    let env = Env {
        store: tmp(tag).join("arbitre.db"),
        vars,
    };
    for m in toggleable(&env) {
        switch(&env, &m.id, true);
    }
    env
}

fn toggleable(env: &Env) -> Vec<ModuleInfo> {
    modules(&env.store)
}

/// `args --json` on `env`.
fn exec(env: &Env, args: &[&str]) -> Run {
    let args = [args, &["--json"]].concat();
    run_timed(&mut on(&env.store, &env.vars, &args)).expect("no hang")
}

fn switch(env: &Env, m: &str, on: bool) {
    let verb = if on { "enable" } else { "disable" };
    let r = exec(env, &["module", verb, "--module", m]);
    assert_eq!(r.code, 0, "{verb} {m}: {}", r.stdout);
}

/// Copy of the store file into a new directory (same device, same data).
fn copy(env: &Env, tag: &str) -> Env {
    let store = tmp(tag).join("arbitre.db");
    std::fs::copy(&env.store, &store).expect("copy store");
    Env {
        store,
        vars: env.vars.clone(),
    }
}

/// Action `a` with its minimal input, through the CLI surface (checks 1 to 6, P-15).
fn call(env: &Env, a: &Json) -> Called {
    Cli.call(env, a["name"].as_str().expect("name"), &input_of(a))
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

fn surface_names(env: &Env) -> Vec<(String, BTreeSet<String>)> {
    surfaces::all()
        .iter()
        .map(|s| (s.name().to_string(), s.actions(env)))
        .collect()
}

/// Writes of namespace `m`, with their positions.
fn namespace(env: &Env, m: &str) -> Vec<(arbitre_core::Position, arbitre_core::Write)> {
    snapshot(&env.store)
        .0
        .into_iter()
        .filter(|(_, w)| w.record.module == m)
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
        switch(&c, &m.id, false);
        for (s, seen) in surface_names(&c) {
            assert!(own.is_disjoint(&seen), "{s}: {} hidden", m.id);
        }
        let before = snapshot(&c.store);
        for a in &m.actions {
            assert_eq!(call(&c, a).code, 4, "{}", a["name"]);
        }
        assert_eq!(snapshot(&c.store), before, "{}: nothing written", m.id);
        switch(&c, &m.id, true);
    }
}

#[test]
fn checks_3_4_data_kept_and_namespace_isolated() {
    let o = Observer::start(Mode::Cut);
    let c = fresh("conf-34", &o);
    for m in toggleable(&c) {
        let after_setup = last_position(&c.store);
        for a in &m.actions {
            call(&c, a);
        }
        for w in modules_written_after(&c.store, after_setup) {
            assert_eq!(w, m.id, "{} wrote outside its namespace", m.id);
        }
        let before = namespace(&c, &m.id);
        switch(&c, &m.id, false);
        switch(&c, &m.id, true);
        assert_eq!(namespace(&c, &m.id), before, "{}: data kept", m.id);
    }
}

#[test]
fn check_5_every_subset_works_alone() {
    let o = Observer::start(Mode::Cut);
    let base = fresh("conf-5", &o);
    let all = toggleable(&base);
    // Same data = same `(record, change)` pairs: stamps differ (`part` has extra writes).
    let data = |e: &Env, m: &str| -> Vec<_> {
        namespace(e, m)
            .into_iter()
            .map(|(_, w)| (w.record, w.change))
            .collect()
    };
    for subset in subsets(all.len()) {
        for m in subset.iter().map(|&i| &all[i]) {
            for a in &m.actions {
                let full = copy(&base, "conf-5-full");
                let part = copy(&base, "conf-5-part");
                for (i, other) in all.iter().enumerate() {
                    if !subset.contains(&i) {
                        switch(&part, &other.id, false);
                    }
                }
                let (x, y) = (call(&full, a), call(&part, a));
                assert_eq!((x.code, &x.doc), (y.code, &y.doc), "{}", a["name"]);
                assert_eq!(data(&full, &m.id), data(&part, &m.id), "{}", a["name"]);
            }
        }
    }
}

#[test]
fn check_6_network_cut() {
    let o = Observer::start(Mode::Cut);
    let c = fresh("conf-6", &o);
    // Matrix US-4 row 6: the foundation actions, cut.
    assert_eq!(exec(&c, &["module", "list"]).code, 0);
    for m in toggleable(&c) {
        switch(&c, &m.id, false);
        switch(&c, &m.id, true);
    }
    assert!(o.take().is_empty(), "foundation actions tried the network");
    for m in toggleable(&c) {
        for a in &m.actions {
            let r = call(&c, a);
            let seen = o.take();
            let declared = !a["requires_network"].as_array().expect("list").is_empty();
            if declared {
                assert!([0, 1, 3].contains(&r.code), "{}: {}", a["name"], r.doc);
                if r.code != 0 {
                    let (_, _, id) = failure(&r.doc);
                    assert!(!id.is_empty());
                }
            } else {
                assert!([0, 1].contains(&r.code), "{}: {}", a["name"], r.doc);
                assert!(seen.is_empty(), "{} tried the network", a["name"]);
            }
        }
    }
}

#[test]
fn check_6_network_open() {
    // RG-ST04-4.4, AC-4.3: every attempt targets a host allowed for a declared kind.
    let o = Observer::start(Mode::Open);
    let c = fresh("conf-6-open", &o);
    assert!(o.take().is_empty(), "foundation actions tried the network");
    for m in toggleable(&c) {
        for a in &m.actions {
            let kinds: Vec<_> = a["requires_network"]
                .as_array()
                .expect("list")
                .iter()
                .map(|k| k.as_str().expect("kind"))
                .collect();
            call(&c, a);
            for at in o.take() {
                let allowed = ALLOWED
                    .iter()
                    .any(|(k, h)| kinds.contains(k) && *h == at.host);
                assert!(allowed, "{}: {}:{}", a["name"], at.host, at.port);
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
                .map(|s| s.call(&copy(&base, "conf-7-call"), name, &input_of(a)))
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
        let r = exec(&c, args);
        assert_eq!(r.code, 0, "{args:?}: {}", r.stdout);
    }
    let r = exec(&c, &["fake-a", "get", "--key", "k"]);
    assert_eq!(r.code, 1, "deleted ⇒ refusal not_found");
    assert!(o.take().is_empty());
}

#[test]
fn subsets_all_up_to_six_then_pairs() {
    // RG-ST04-5.2 item 5: nothing dropped, no shift overflow past 32 modules.
    assert_eq!(subsets(3).len(), 7);
    assert_eq!(subsets(6).len(), 63);
    let big = subsets(40);
    assert_eq!(big.len(), 40 + 40 * 39 / 2);
    assert!(big.contains(&vec![39]) && big.contains(&vec![38, 39]));
    assert!(big.iter().all(|s| s.len() <= 2));
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
