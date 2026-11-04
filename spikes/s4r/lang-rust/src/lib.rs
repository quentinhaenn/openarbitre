//! Rust adapter on tree-sitter-rust. File -> module from the crate layout; `use` trees expanded.

use facts::{Extraction, Extractor, Fact, Kind, Lang, Location, QName, SourceFile, Visibility};
use std::collections::{BTreeSet, HashMap};
use tree_sitter::{Node, Parser};

pub struct Rust;

const LANG: Lang = "rust";

impl Extractor for Rust {
    fn language(&self) -> Lang {
        LANG
    }
    fn separator(&self) -> &'static str {
        "::"
    }
    fn handles(&self, path: &str) -> bool {
        path.ends_with(".rs")
    }
    fn extract(&self, snapshot: &[SourceFile]) -> Extraction {
        let crates = crate_dirs(snapshot);
        let files: Vec<(&SourceFile, Vec<String>)> = snapshot
            .iter()
            .filter(|f| self.handles(&f.path))
            .filter_map(|f| file_module(&f.path, &crates).map(|m| (f, m)))
            .collect();
        let known: BTreeSet<Vec<String>> = files.iter().map(|(_, m)| m.clone()).collect();
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).expect("grammar ABI");
        let mut out = Extraction::default();
        for (f, module) in files {
            let tree = parser.parse(&f.text, None).expect("no timeout set");
            let root = tree.root_node();
            if root.has_error() {
                out.invalid.push(loc(&f.path, first_error(root)));
            }
            let crate_name = module[0].clone();
            let mut cx = Cx { file: f, known: &known, crate_name, aliases: HashMap::new(), out: &mut out.facts, parser: &mut parser };
            cx.push(root, &module, Kind::Module);
            let mut module = module;
            cx.collect_aliases(root, &module);
            cx.visit(root, &mut module, true);
        }
        out
    }
}

/// Directory -> crate name, from each `Cargo.toml` (`[lib] name` wins over `[package] name`).
pub fn crate_dirs(snapshot: &[SourceFile]) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for f in snapshot.iter().filter(|f| f.path == "Cargo.toml" || f.path.ends_with("/Cargo.toml")) {
        let (mut section, mut name) = ("", None);
        for l in f.text.lines().map(str::trim) {
            if l.starts_with('[') {
                section = l;
            } else if (section == "[package]" || section == "[lib]") && l.starts_with("name")
                && let Some((_, val)) = l.split_once('=') {
                    let val = val.trim().trim_matches('"').replace('-', "_");
                    if section == "[lib]" || name.is_none() {
                        name = Some(val);
                    }
                }
        }
        if let Some(n) = name {
            v.push((f.path.trim_end_matches("Cargo.toml").to_string(), n));
        }
    }
    v.sort_by_key(|(d, _)| std::cmp::Reverse(d.len()));
    v
}

// ponytail: path-derived module tree; ignores `#[path]`, `include!` and cfg-gated `mod`; follow `mod` decls if that bites.
pub fn file_module(path: &str, crates: &[(String, String)]) -> Option<Vec<String>> {
    let (dir, name) = crates.iter().find(|(d, _)| path.starts_with(d.as_str()))?;
    let rel = &path[dir.len()..];
    let mut segs: Vec<String> = vec![name.clone()];
    let rest = match rel.strip_prefix("src/") {
        Some(r) => match r.strip_prefix("bin/") {
            Some(b) => {
                segs = vec![]; // a binary is its own crate, named after the file or dir
                b.replace("/main.rs", ".rs")
            }
            None => r.to_string(),
        },
        // tests/, benches/, examples/, build.rs are other crates: `arbitre_core#tests::hlc`.
        None => {
            let (kind, r) = rel.split_once('/').unwrap_or((rel.trim_end_matches(".rs"), ""));
            segs = vec![format!("{name}#{kind}")];
            r.to_string()
        }
    };
    let rest = rest.trim_end_matches(".rs");
    let parts: Vec<&str> = rest.split('/').collect();
    for (i, p) in parts.iter().enumerate() {
        let last = i + 1 == parts.len();
        if p.is_empty() || last && (*p == "mod" || (segs.len() == 1 && i == 0 && (*p == "lib" || *p == "main"))) {
            continue;
        }
        segs.push((*p).to_string());
    }
    Some(segs)
}

fn first_error(n: Node) -> Node {
    if n.is_error() || n.is_missing() {
        return n;
    }
    let mut c = n.walk();
    let found = n.children(&mut c).find(|ch| ch.has_error()).map(first_error);
    found.unwrap_or(n)
}

fn loc(path: &str, n: Node) -> Location {
    let p = n.start_position();
    Location { path: path.to_string(), line: p.row as u32 + 1, column: p.column as u32 + 1, byte: n.start_byte() as u32 }
}

fn segs_of(text: &str) -> Vec<String> {
    text.split("::").map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()
}

fn visibility(n: Node, src: &str) -> Visibility {
    let mut c = n.walk();
    let vis = n.children(&mut c).find(|ch| ch.kind() == "visibility_modifier");
    match vis.map(|v| &src[v.byte_range()]) {
        None => Visibility::Private,
        Some("pub") => Visibility::Public,
        Some(_) => Visibility::Restricted,
    }
}

struct Cx<'a> {
    file: &'a SourceFile,
    known: &'a BTreeSet<Vec<String>>,
    crate_name: String,
    /// Local name -> resolved path, from this file's `use` (ponytail: file-wide, not block-scoped).
    aliases: HashMap<String, Vec<String>>,
    out: &'a mut Vec<Fact>,
    parser: &'a mut Parser,
}

impl Cx<'_> {
    fn text(&self, n: Node) -> &str {
        &self.file.text[n.byte_range()]
    }

    fn push(&mut self, n: Node, module: &[String], kind: Kind) {
        let module = QName { lang: LANG, segments: module.to_vec() };
        self.out.push(Fact { at: loc(&self.file.path, n), module, kind });
    }

    /// `crate`/`self`/`super`/`$crate`, local aliases, child modules; otherwise left as written (extern crate).
    fn resolve(&self, segs: Vec<String>, module: &[String], use_aliases: bool) -> Vec<String> {
        let Some(first) = segs.first() else { return segs };
        match first.as_str() {
            "crate" | "$crate" => [vec![self.crate_name.clone()], segs[1..].to_vec()].concat(),
            "self" => [module.to_vec(), segs[1..].to_vec()].concat(),
            "super" => {
                let ups = segs.iter().take_while(|s| *s == "super").count();
                let base = &module[..module.len().saturating_sub(ups)];
                [base.to_vec(), segs[ups..].to_vec()].concat()
            }
            _ => {
                if use_aliases && let Some(p) = self.aliases.get(first) {
                    return [p.clone(), segs[1..].to_vec()].concat();
                }
                let local = [module.to_vec(), vec![first.clone()]].concat();
                if self.known.contains(&local) { [module.to_vec(), segs].concat() } else { segs }
            }
        }
    }

    /// Expands a use tree into (path, local name, leaf node).
    fn expand<'t>(&self, n: Node<'t>, prefix: &[String], acc: &mut Vec<(Vec<String>, Option<String>, Node<'t>)>) {
        let join = |p: &[String], t: &str| [p.to_vec(), segs_of(t)].concat();
        match n.kind() {
            "use_list" => {
                let mut c = n.walk();
                for ch in n.named_children(&mut c) {
                    self.expand(ch, prefix, acc);
                }
            }
            "scoped_use_list" => {
                let p = n.child_by_field_name("path").map_or(prefix.to_vec(), |p| join(prefix, self.text(p)));
                if let Some(l) = n.child_by_field_name("list") {
                    self.expand(l, &p, acc);
                }
            }
            "use_as_clause" => {
                let mut p = join(prefix, self.text(n.child_by_field_name("path").unwrap_or(n)));
                if p.last().is_some_and(|s| s == "self") {
                    p.pop(); // `use a::{self as b}`
                }
                let alias = n.child_by_field_name("alias").map(|a| self.text(a).to_string());
                acc.push((p, alias, n));
            }
            "use_wildcard" => {
                let mut c = n.walk();
                let inner = n.named_children(&mut c).next();
                let p = inner.map_or(prefix.to_vec(), |i| join(prefix, self.text(i)));
                acc.push((p, Some("*".into()), n));
            }
            _ => {
                let mut p = join(prefix, self.text(n));
                if p.last().is_some_and(|s| s == "self") {
                    p.pop(); // `use a::{self}` imports `a`
                }
                let name = p.last().cloned();
                acc.push((p, name, n));
            }
        }
    }

    /// Module-level `m! { items }` (tokio's `cfg_rt! { mod x; use y; }`): re-parse the body in place,
    /// locations stay exact; kept only if it parses as items without error.
    fn items_in_macro(&mut self, n: Node, module: &mut Vec<String>) {
        let mut c = n.walk();
        let Some(tt) = n.named_children(&mut c).filter(|x| x.kind() == "token_tree").last() else { return };
        if !self.text(tt).starts_with('{') {
            return;
        }
        let (s, e) = (tt.start_position(), tt.end_position());
        let inner = tree_sitter::Range {
            start_byte: tt.start_byte() + 1,
            end_byte: tt.end_byte() - 1,
            start_point: tree_sitter::Point { row: s.row, column: s.column + 1 },
            end_point: tree_sitter::Point { row: e.row, column: e.column - 1 },
        };
        let text = self.file.text.as_str();
        let sub = self.parser.set_included_ranges(&[inner]).ok().and_then(|_| self.parser.parse(text, None));
        self.parser.set_included_ranges(&[]).expect("reset");
        if let Some(sub) = sub.filter(|t| !t.root_node().has_error()) {
            self.visit(sub.root_node(), module, true);
        }
    }

    fn collect_aliases(&mut self, root: Node, module: &[String]) {
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if n.kind() == "use_declaration" {
                let mut acc = Vec::new();
                if let Some(arg) = n.child_by_field_name("argument") {
                    self.expand(arg, &[], &mut acc);
                }
                for (p, name, _) in acc {
                    if let Some(name) = name.filter(|n| n != "*" && n != "_") {
                        let r = self.resolve(p, module, false);
                        self.aliases.insert(name, r);
                    }
                }
                continue;
            }
            let mut c = n.walk();
            stack.extend(n.named_children(&mut c));
        }
    }

    fn visit(&mut self, n: Node, module: &mut Vec<String>, module_level: bool) {
        let kind = n.kind();
        match kind {
            "mod_item" => {
                let name = n.child_by_field_name("name").map(|x| self.text(x).to_string()).unwrap_or_default();
                self.push(n, module, Kind::Symbol(name.clone(), visibility(n, &self.file.text)));
                if let Some(body) = n.child_by_field_name("body") {
                    module.push(name);
                    self.push(n, module, Kind::Module);
                    self.visit(body, module, true);
                    module.pop();
                }
                return;
            }
            "use_declaration" => {
                let mut acc = Vec::new();
                if let Some(arg) = n.child_by_field_name("argument") {
                    self.expand(arg, &[], &mut acc);
                }
                let vis = visibility(n, &self.file.text);
                for (p, name, leaf) in acc {
                    let target = QName { lang: LANG, segments: self.resolve(p, module, false) };
                    self.push(leaf, module, Kind::Depends(target));
                    if vis != Visibility::Private && let Some(name) = name {
                        self.push(leaf, module, Kind::Symbol(name, vis)); // re-export
                    }
                }
                return;
            }
            "extern_crate_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    let target = QName { lang: LANG, segments: vec![self.text(name).to_string()] };
                    self.push(n, module, Kind::Depends(target));
                }
                return;
            }
            "function_item" | "struct_item" | "enum_item" | "union_item" | "trait_item" | "type_item"
            | "const_item" | "static_item"
                if module_level =>
            {
                if let Some(name) = n.child_by_field_name("name") {
                    let s = Kind::Symbol(self.text(name).to_string(), visibility(n, &self.file.text));
                    self.push(name, module, s);
                }
            }
            "call_expression" | "macro_invocation" => {
                let f = n.child_by_field_name(if kind == "call_expression" { "function" } else { "macro" });
                let f = f.map(|f| if f.kind() == "generic_function" { f.child_by_field_name("function").unwrap_or(f) } else { f });
                if let Some(f) = f.filter(|f| matches!(f.kind(), "identifier" | "scoped_identifier")) {
                    let mut segs = segs_of(self.text(f));
                    if kind == "macro_invocation" && let Some(last) = segs.last_mut() {
                        last.push('!');
                    }
                    let callee = QName { lang: LANG, segments: self.resolve(segs, module, true) };
                    self.push(f, module, Kind::Call(callee));
                }
                if kind == "macro_invocation" && module_level {
                    self.items_in_macro(n, module);
                    return;
                }
            }
            "type_identifier" | "scoped_type_identifier" => {
                let parent = n.parent().map(|p| p.kind()).unwrap_or("");
                let is_def = n.parent().and_then(|p| p.child_by_field_name("name")).is_some_and(|x| x.id() == n.id())
                    && parent != "scoped_type_identifier";
                if !is_def && parent != "scoped_type_identifier" {
                    let t = QName { lang: LANG, segments: self.resolve(segs_of(self.text(n)), module, true) };
                    self.push(n, module, Kind::TypeRef(t));
                }
                return;
            }
            _ => {}
        }
        let mut c = n.walk();
        let level = kind == "source_file" || (kind == "declaration_list" && module_level);
        for ch in n.named_children(&mut c) {
            self.visit(ch, module, level);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(p: &str, t: &str) -> SourceFile {
        SourceFile { path: p.into(), text: t.into() }
    }

    #[test]
    fn use_trees_modules_visibility() {
        let snap = [
            file("crates/core/Cargo.toml", "[package]\nname = \"arbitre-core\"\n"),
            file("crates/core/src/lib.rs", "pub mod store;\nmod inner { pub(crate) fn f() {} }\n"),
            file(
                "crates/core/src/store/mod.rs",
                "use super::inner::f;\nuse std::{io::{self, Read as R}, fmt::*};\nuse rusqlite::Connection;\npub use crate::inner;\nextern crate std;\npub fn g(c: Connection) { f(); io::stdout(); }\n",
            ),
        ];
        let x = Rust.extract(&snap);
        let deps: Vec<String> = x
            .facts
            .iter()
            .filter_map(|f| match &f.kind {
                Kind::Depends(t) => Some(format!("{}@{}:{}", t.segments.join("::"), f.at.line, f.at.column)),
                _ => None,
            })
            .collect();
        assert_eq!(
            deps,
            [
                "arbitre_core::inner::f@1:5",
                "std::io@2:16",
                "std::io::Read@2:22",
                "std::fmt@2:34",
                "rusqlite::Connection@3:5",
                "arbitre_core::inner@4:9",
                "std@5:1"
            ]
        );
        let has = |k: Kind| x.facts.iter().any(|f| f.kind == k);
        assert!(has(Kind::Symbol("f".into(), Visibility::Restricted)));
        assert!(has(Kind::Symbol("inner".into(), Visibility::Public))); // pub use re-export
        assert!(has(Kind::Call(QName { lang: LANG, segments: segs_of("arbitre_core::inner::f") })));
        assert!(has(Kind::Call(QName { lang: LANG, segments: segs_of("std::io::stdout") })));
        assert!(has(Kind::TypeRef(QName { lang: LANG, segments: segs_of("rusqlite::Connection") })));
        assert!(x.invalid.is_empty());
    }

    #[test]
    fn items_inside_module_level_macro() {
        let snap = [file("Cargo.toml", "[package]\nname = \"t\"\n"), file("src/lib.rs", "cfg_rt! {\n    use rusqlite::C;\n}\n")];
        let x = Rust.extract(&snap);
        let dep = x.facts.iter().find(|f| matches!(f.kind, Kind::Depends(_))).unwrap();
        assert_eq!((dep.at.line, dep.at.column, dep.at.byte), (2, 9, 18));
    }
}
