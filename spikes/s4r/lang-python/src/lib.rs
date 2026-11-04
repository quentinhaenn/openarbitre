//! Python adapter on tree-sitter-python: modules, imports (relative resolved), symbols with `__all__`.

use facts::{Extraction, Extractor, Fact, Kind, Lang, Location, QName, SourceFile, Visibility};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

pub struct Python;

const LANG: Lang = "python";

impl Extractor for Python {
    fn language(&self) -> Lang {
        LANG
    }
    fn separator(&self) -> &'static str {
        "."
    }
    fn handles(&self, path: &str) -> bool {
        path.ends_with(".py")
    }
    fn extract(&self, snapshot: &[SourceFile]) -> Extraction {
        let inits: HashSet<&str> =
            snapshot.iter().filter_map(|f| f.path.strip_suffix("/__init__.py")).collect();
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).expect("grammar ABI");
        let mut out = Extraction::default();
        for f in snapshot.iter().filter(|f| self.handles(&f.path)) {
            let (module, is_pkg) = module_of(&f.path, &inits);
            let tree = parser.parse(&f.text, None).expect("no timeout set");
            let root = tree.root_node();
            if root.has_error() {
                out.invalid.push(loc(&f.path, root));
            }
            let m = QName { lang: LANG, segments: module.clone() };
            let push = |out: &mut Vec<Fact>, n: Node, kind| out.push(Fact { at: loc(&f.path, n), module: m.clone(), kind });
            push(&mut out.facts, root, Kind::Module);
            let text = |n: Node| f.text[n.byte_range()].to_string();
            // Package that relative imports start from.
            let package = if is_pkg { module.clone() } else { module[..module.len().saturating_sub(1)].to_vec() };
            let mut all: Option<Vec<String>> = None;
            let mut defs: Vec<(String, Node)> = Vec::new();
            let mut stack = vec![root];
            while let Some(n) = stack.pop() {
                let mut c = n.walk();
                match n.kind() {
                    "import_statement" => {
                        for name in n.children_by_field_name("name", &mut c) {
                            let dotted = name.child_by_field_name("name").unwrap_or(name);
                            push(&mut out.facts, name, Kind::Depends(q(dotted_segs(&text(dotted)))));
                        }
                    }
                    "import_from_statement" => {
                        let Some(src) = n.child_by_field_name("module_name") else { continue };
                        let base = if src.kind() == "relative_import" {
                            let raw = text(src);
                            let level = raw.chars().take_while(|ch| *ch == '.').count();
                            let up = package.len().saturating_sub(level - 1);
                            [package[..up].to_vec(), dotted_segs(&raw[level..])].concat()
                        } else {
                            dotted_segs(&text(src))
                        };
                        let names: Vec<Node> = n.children_by_field_name("name", &mut c).collect();
                        if names.is_empty() {
                            push(&mut out.facts, src, Kind::Depends(q(base.clone()))); // `import *`
                        }
                        for name in names {
                            let dotted = name.child_by_field_name("name").unwrap_or(name);
                            let target = [base.clone(), dotted_segs(&text(dotted))].concat();
                            push(&mut out.facts, name, Kind::Depends(q(target)));
                        }
                    }
                    "module" => stack.extend(n.named_children(&mut c).collect::<Vec<_>>().into_iter().rev()),
                    // Module level only; nested imports in functions are still found below.
                    "function_definition" | "class_definition" if n.parent().is_some_and(|p| p.kind() == "module") => {
                        if let Some(name) = n.child_by_field_name("name") {
                            defs.push((text(name), name));
                        }
                        stack.extend(n.named_children(&mut c));
                    }
                    "expression_statement" if n.parent().is_some_and(|p| p.kind() == "module") => {
                        let a = n.named_child(0).filter(|a| a.kind() == "assignment");
                        if let Some(left) = a.and_then(|a| a.child_by_field_name("left")).filter(|l| l.kind() == "identifier") {
                            let name = text(left);
                            if name == "__all__" {
                                let right = a.and_then(|a| a.child_by_field_name("right"));
                                all = right.map(|r| strings(r, &f.text));
                            } else {
                                defs.push((name, left));
                            }
                        }
                    }
                    _ => stack.extend(n.named_children(&mut c)),
                }
            }
            for (name, node) in defs {
                let v = match &all {
                    Some(all) if all.contains(&name) => Visibility::Public,
                    Some(_) => Visibility::Private,
                    None if name.starts_with('_') => Visibility::Private,
                    None => Visibility::Public,
                };
                push(&mut out.facts, node, Kind::Symbol(name, v));
            }
        }
        out
    }
}

fn q(segments: Vec<String>) -> QName {
    QName { lang: LANG, segments }
}

fn dotted_segs(s: &str) -> Vec<String> {
    s.split('.').map(str::trim).filter(|x| !x.is_empty()).map(str::to_string).collect()
}

/// String literals of a list/tuple (`__all__`), quotes stripped.
fn strings(n: Node, src: &str) -> Vec<String> {
    let mut c = n.walk();
    n.named_children(&mut c)
        .filter(|s| s.kind() == "string")
        .map(|s| src[s.byte_range()].trim_matches(|ch| ch == '"' || ch == '\'').to_string())
        .collect()
}

/// `a/b/c.py` -> `b.c` when `a/` has no `__init__.py` and `a/b/` has one; `__init__.py` names its package.
// ponytail: package roots inferred from `__init__.py`; namespace packages (PEP 420) need configured roots.
fn module_of(path: &str, inits: &HashSet<&str>) -> (Vec<String>, bool) {
    let is_pkg = path.ends_with("/__init__.py") || path == "__init__.py";
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    let mut segs: Vec<String> = Vec::new();
    if !is_pkg {
        let stem = path.rsplit('/').next().unwrap_or(path).trim_end_matches(".py");
        segs.push(stem.to_string());
    }
    let mut d = dir;
    while !d.is_empty() && inits.contains(d) {
        let (parent, name) = d.rsplit_once('/').unwrap_or(("", d));
        segs.push(name.to_string());
        d = parent;
    }
    segs.reverse();
    (segs, is_pkg)
}

fn loc(path: &str, n: Node) -> Location {
    let p = n.start_position();
    Location { path: path.to_string(), line: p.row as u32 + 1, column: p.column as u32 + 1, byte: n.start_byte() as u32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(p: &str, t: &str) -> SourceFile {
        SourceFile { path: p.into(), text: t.into() }
    }

    #[test]
    fn relative_imports_and_all() {
        let snap = [
            file("src/app/__init__.py", "from .core import api\n"),
            file("src/app/core/__init__.py", ""),
            file(
                "src/app/core/api.py",
                "import os.path as p, json\nfrom ..db import conn\nfrom . import models\nfrom .x import *\n__all__ = ['run']\ndef run(): pass\ndef helper(): import sqlite3\n",
            ),
        ];
        let x = Python.extract(&snap);
        let deps: Vec<String> = x
            .facts
            .iter()
            .filter_map(|f| match &f.kind {
                Kind::Depends(t) => Some(format!("{}<-{}@{}:{}", t.segments.join("."), f.module.segments.join("."), f.at.line, f.at.column)),
                _ => None,
            })
            .collect();
        assert_eq!(
            deps,
            [
                "app.core.api<-app@1:19",
                "os.path<-app.core.api@1:8",
                "json<-app.core.api@1:22",
                "app.db.conn<-app.core.api@2:18",
                "app.core.models<-app.core.api@3:15",
                "app.core.x<-app.core.api@4:6",
                "sqlite3<-app.core.api@7:22",
            ]
        );
        let has = |k: Kind| x.facts.iter().any(|f| f.kind == k);
        assert!(has(Kind::Symbol("run".into(), Visibility::Public)));
        assert!(has(Kind::Symbol("helper".into(), Visibility::Private))); // excluded by __all__
    }
}
