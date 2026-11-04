//! Comparison adapter on `syn` + `proc-macro2` spans: modules, `use`/`extern crate`, symbols only.
//! Shares the crate layout logic with `lang-rust` to compare parsers, not layouts.

use facts::{Extraction, Extractor, Fact, Kind, Lang, Location, QName, SourceFile, Visibility};
use proc_macro2::Span;
use std::collections::BTreeSet;
use syn::visit::Visit;

pub struct RustSyn;

const LANG: Lang = "rust";

impl Extractor for RustSyn {
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
        let crates = lang_rust::crate_dirs(snapshot);
        let files: Vec<(&SourceFile, Vec<String>)> = snapshot
            .iter()
            .filter(|f| self.handles(&f.path))
            .filter_map(|f| lang_rust::file_module(&f.path, &crates).map(|m| (f, m)))
            .collect();
        let known: BTreeSet<Vec<String>> = files.iter().map(|(_, m)| m.clone()).collect();
        let mut out = Extraction::default();
        for (f, module) in files {
            match syn::parse_file(&f.text) {
                Err(e) => out.invalid.push(at(f, e.span())), // all-or-nothing: no facts for this file
                Ok(ast) => {
                    let crate_name = module[0].clone();
                    let mut v = V { f, known: &known, crate_name, module, out: &mut out.facts };
                    let top = Location { path: f.path.clone(), line: 1, column: 1, byte: 0 };
                    v.out.push(Fact { at: top, module: v.q(v.module.clone()), kind: Kind::Module });
                    v.visit_file(&ast);
                }
            }
            // Spans live in a thread-local source map that only grows; reset it per file.
            proc_macro2::extra::invalidate_current_thread_spans();
        }
        out
    }
}

fn at(f: &SourceFile, s: Span) -> Location {
    let byte = s.byte_range().start;
    let line_start = f.text[..byte].rfind('\n').map_or(0, |i| i + 1);
    Location { path: f.path.clone(), line: s.start().line as u32, column: (byte - line_start) as u32 + 1, byte: byte as u32 }
}

fn vis(v: &syn::Visibility) -> Visibility {
    match v {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(_) => Visibility::Restricted,
        syn::Visibility::Inherited => Visibility::Private,
    }
}

struct V<'a> {
    f: &'a SourceFile,
    known: &'a BTreeSet<Vec<String>>,
    crate_name: String,
    module: Vec<String>,
    out: &'a mut Vec<Fact>,
}

impl V<'_> {
    fn q(&self, segments: Vec<String>) -> QName {
        QName { lang: LANG, segments }
    }

    // Same rules as lang-rust's resolve (aliases not needed for `use` targets).
    fn resolve(&self, segs: Vec<String>) -> Vec<String> {
        let m = &self.module;
        match segs.first().map(String::as_str) {
            Some("crate") => [vec![self.crate_name.clone()], segs[1..].to_vec()].concat(),
            Some("self") => [m.clone(), segs[1..].to_vec()].concat(),
            Some("super") => {
                let ups = segs.iter().take_while(|s| *s == "super").count();
                [m[..m.len().saturating_sub(ups)].to_vec(), segs[ups..].to_vec()].concat()
            }
            Some(first) if self.known.contains(&[m.clone(), vec![first.to_string()]].concat()) => [m.clone(), segs].concat(),
            _ => segs,
        }
    }

    fn dep(&mut self, mut path: Vec<String>, start: Span, name: Option<String>, v: Visibility) {
        if path.last().is_some_and(|s| s == "self") {
            path.pop();
        }
        let name = name.or_else(|| path.last().cloned());
        let target = self.q(self.resolve(path));
        let (at, module) = (at(self.f, start), self.q(self.module.clone()));
        self.out.push(Fact { at: at.clone(), module: module.clone(), kind: Kind::Depends(target) });
        if v != Visibility::Private && let Some(name) = name {
            self.out.push(Fact { at, module, kind: Kind::Symbol(name, v) });
        }
    }

    fn tree(&mut self, t: &syn::UseTree, prefix: &mut Vec<String>, start: Option<Span>, v: Visibility) {
        use syn::UseTree::*;
        match t {
            Path(p) => {
                prefix.push(p.ident.to_string());
                self.tree(&p.tree, prefix, start.or(Some(p.ident.span())), v);
                prefix.pop();
            }
            Name(n) => {
                let path = [prefix.clone(), vec![n.ident.to_string()]].concat();
                self.dep(path, start.unwrap_or(n.ident.span()), None, v);
            }
            Rename(r) => {
                let path = [prefix.clone(), vec![r.ident.to_string()]].concat();
                self.dep(path, start.unwrap_or(r.ident.span()), Some(r.rename.to_string()), v);
            }
            Glob(g) => self.dep(prefix.clone(), start.unwrap_or(g.star_token.span), Some("*".into()), v),
            Group(g) => {
                for t in &g.items {
                    self.tree(t, prefix, None, v);
                }
            }
        }
    }

    fn sym(&mut self, ident: &syn::Ident, v: &syn::Visibility) {
        let fact = Fact { at: at(self.f, ident.span()), module: self.q(self.module.clone()), kind: Kind::Symbol(ident.to_string(), vis(v)) };
        self.out.push(fact);
    }
}

impl<'ast> Visit<'ast> for V<'_> {
    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        self.tree(&i.tree, &mut Vec::new(), None, vis(&i.vis));
    }
    fn visit_item_extern_crate(&mut self, i: &'ast syn::ItemExternCrate) {
        let at = at(self.f, i.extern_token.span);
        let target = self.q(vec![i.ident.to_string()]);
        self.out.push(Fact { at, module: self.q(self.module.clone()), kind: Kind::Depends(target) });
    }
    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        self.sym(&i.ident, &i.vis);
        if let Some((_, items)) = &i.content {
            self.module.push(i.ident.to_string());
            let fact = Fact { at: at(self.f, i.mod_token.span), module: self.q(self.module.clone()), kind: Kind::Module };
            self.out.push(fact);
            for it in items {
                self.visit_item(it);
            }
            self.module.pop();
        }
    }
}
