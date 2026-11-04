//! `demo check <dir> <rules>`   : extract with every registered adapter, apply the `!->` rules of the file.
//! `demo bench <dir>`           : tree-sitter vs syn on the Rust files (parse time, facts agreement).

use facts::{Extractor, Forbid, Kind, Registry, SourceFile, check};
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

fn registry() -> Registry {
    Registry::new(vec![
        Box::new(lang_rust::Rust),
        Box::new(lang_python::Python), // the whole "add a language" change on the core side
    ])
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, dir) = (args[0].as_str(), args[1].as_str());
    let snap = load(dir);
    match mode {
        "check" => {
            let rules = std::fs::read_to_string(&args[2]).unwrap();
            let rules: Vec<String> = rules.lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#')).map(Into::into).collect();
            run_check(&snap, &rules)
        }
        "bench" => bench(&snap),
        _ => panic!("check | bench"),
    }
}

/// Snapshot = every UTF-8 file under `dir`, paths relative, sorted; skips `target/` and dot dirs.
fn load(dir: &str) -> Vec<SourceFile> {
    let mut out = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(dir)];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if !name.starts_with('.') && name != "target" {
                    stack.push(p);
                }
            } else if let Ok(text) = std::fs::read_to_string(&p) {
                let path = p.strip_prefix(dir).unwrap().to_string_lossy().trim_start_matches('/').to_string();
                out.push(SourceFile { path, text });
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

fn run_check(snap: &[SourceFile], rules: &[String]) {
    let reg = registry();
    let t = Instant::now();
    let report = reg.extract(snap);
    let took = t.elapsed();
    let rules: Vec<Forbid> = rules.iter().map(|r| Forbid::parse(r, &reg).unwrap()).collect();
    let by_lang = |l: &str| report.facts.iter().filter(|f| f.module.lang == l).count();
    println!(
        "files {} | facts rust {} python {} | invalid {} | unchecked {} | extract {:?}",
        snap.len(),
        by_lang("rust"),
        by_lang("python"),
        report.invalid.len(),
        report.unchecked.len(),
        took
    );
    for loc in report.invalid.iter().take(5) {
        println!("invalid {}:{}:{}", loc.path, loc.line, loc.column);
    }
    let violations = check(&report.facts, &rules);
    for (i, f) in &violations {
        let Kind::Depends(target) = &f.kind else { unreachable!() };
        println!(
            "VIOLATION rule {} {}:{}:{} {} -> {}",
            i,
            f.at.path,
            f.at.line,
            f.at.column,
            reg.render(&f.module),
            reg.render(target)
        );
    }
    println!("violations {}", violations.len());
    // Determinism: reversed and rotated file order give the same facts.
    let mut rev: Vec<SourceFile> = snap.iter().rev().map(|f| SourceFile { path: f.path.clone(), text: f.text.clone() }).collect();
    rev.rotate_left(snap.len() / 3);
    assert!(reg.extract(&rev).facts == report.facts, "non-deterministic");
    println!("determinism ok (permuted file order)");
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn bench(snap: &[SourceFile]) {
    let rs: Vec<&SourceFile> = snap.iter().filter(|f| f.path.ends_with(".rs")).collect();
    let lines: usize = rs.iter().map(|f| f.text.lines().count()).sum();
    let bytes: usize = rs.iter().map(|f| f.text.len()).sum();
    println!("rust files {} lines {} bytes {}", rs.len(), lines, bytes);

    let mut p = tree_sitter::Parser::new();
    p.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let (mut ts, mut sy, mut ts_err, mut syn_err) = (vec![], vec![], 0, 0);
    for _ in 0..5 {
        let t = Instant::now();
        ts_err = rs.iter().filter(|f| p.parse(&f.text, None).unwrap().root_node().has_error()).count();
        ts.push(t.elapsed());
        let t = Instant::now();
        syn_err = rs.iter().filter(|f| syn::parse_file(&f.text).is_err()).count();
        sy.push(t.elapsed());
    }
    // `use` / `mod` lines that sit inside a macro invocation (cfg_*! { .. }): invisible to both parsers.
    let (mut hidden_use, mut hidden_mod, mut all_use) = (0, 0, 0);
    for f in &rs {
        let tree = p.parse(&f.text, None).unwrap();
        let mut ranges = vec![];
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            if n.kind() == "macro_invocation" || n.kind() == "macro_definition" {
                ranges.push(n.byte_range());
                continue;
            }
            let mut c = n.walk();
            stack.extend(n.named_children(&mut c));
        }
        let mut off = 0;
        for l in f.text.split_inclusive('\n') {
            let t = l.trim_start();
            let t = t.strip_prefix("pub(crate) ").or(t.strip_prefix("pub ")).unwrap_or(t);
            let inside = ranges.iter().any(|r| r.contains(&off));
            if t.starts_with("use ") {
                all_use += 1;
                hidden_use += inside as usize;
            }
            if t.starts_with("mod ") && inside {
                hidden_mod += 1;
            }
            off += l.len();
        }
    }
    println!("use lines {all_use}, inside macro invocations/definitions {hidden_use}; mod lines inside macros {hidden_mod}");
    println!("parse only, median of 5: tree-sitter {:?} ({ts_err} files with errors) | syn {:?} ({syn_err} files rejected)", median(ts), median(sy));

    let (mut te, mut se) = (vec![], vec![]);
    let (mut a, mut b) = (Default::default(), Default::default());
    for _ in 0..5 {
        let t = Instant::now();
        a = lang_rust::Rust.extract(snap);
        te.push(t.elapsed());
        let t = Instant::now();
        b = lang_rust_syn::RustSyn.extract(snap);
        se.push(t.elapsed());
    }
    println!("extract, median of 5: tree-sitter (all facts) {:?} | syn (deps + symbols of mods) {:?}", median(te), median(se));
    let deps = |x: &facts::Extraction| -> BTreeSet<(String, u32, String, String)> {
        x.facts
            .iter()
            .filter_map(|f| match &f.kind {
                Kind::Depends(t) => Some((f.at.path.clone(), f.at.line, f.module.segments.join("::"), t.segments.join("::"))),
                _ => None,
            })
            .collect()
    };
    let (da, db) = (deps(&a), deps(&b));
    let same_cols = a
        .facts
        .iter()
        .filter(|f| matches!(f.kind, Kind::Depends(_)))
        .filter(|f| b.facts.iter().any(|g| g.kind == f.kind && g.at == f.at))
        .count();
    println!(
        "depends facts: tree-sitter {} syn {} | common {} | only ts {} | only syn {} | identical location (line,col,byte) {}",
        da.len(),
        db.len(),
        da.intersection(&db).count(),
        da.difference(&db).count(),
        db.difference(&da).count(),
        same_cols
    );
    let moved = a.facts.iter().filter(|f| matches!(f.kind, Kind::Depends(_)) && !b.facts.iter().any(|g| g.kind == f.kind && g.at == f.at));
    for f in moved.take(3) {
        println!("  location differs: {}:{}:{} {:?}", f.at.path, f.at.line, f.at.column, f.kind);
    }
    for d in da.difference(&db).take(4) {
        println!("  only ts : {d:?}");
    }
    for d in db.difference(&da).take(4) {
        println!("  only syn: {d:?}");
    }
    let kinds = |k: fn(&Kind) -> bool| a.facts.iter().filter(|f| k(&f.kind)).count();
    println!(
        "tree-sitter facts: modules {} symbols {} calls {} typerefs {} | invalid files ts {} syn {}",
        kinds(|k| matches!(k, Kind::Module)),
        kinds(|k| matches!(k, Kind::Symbol(..))),
        kinds(|k| matches!(k, Kind::Call(_))),
        kinds(|k| matches!(k, Kind::TypeRef(_))),
        a.invalid.len(),
        b.invalid.len()
    );
}
