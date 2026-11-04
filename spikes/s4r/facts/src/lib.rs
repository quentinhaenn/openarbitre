//! Spike S4-R: language-neutral `Extractor` port, fact model, registry and a toy checker.
//! Nothing here knows any language. See `.ctdd/spikes/S4-R.md`.

/// One file of the snapshot under check (base or head).
pub struct SourceFile {
    pub path: String,
    pub text: String,
}

/// Language tag, owned by the adapter (`"rust"`, `"python"`).
pub type Lang = &'static str;

/// A qualified name in one language: a module, an import target, a callee, a type.
/// `segments` are already resolved by the adapter (no `crate`/`super`/leading dots left).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QName {
    pub lang: Lang,
    pub segments: Vec<String>,
}

impl QName {
    /// Segment-prefix match: `a::b` covers `a::b` and `a::b::c`, not `a::bc`.
    pub fn starts_with(&self, prefix: &QName) -> bool {
        self.lang == prefix.lang && self.segments.starts_with(&prefix.segments)
    }
}

/// Exact location: 1-based line, 1-based column counted in bytes, 0-based byte offset.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub byte: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility {
    /// Part of the module's exported surface (`pub`, listed in `__all__`, no leading `_`).
    Public,
    /// Visible beyond the module but not exported (`pub(crate)`, `pub(super)`, `pub(in ..)`).
    Restricted,
    Private,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// The module exists; `at` is where it starts.
    Module,
    /// Import / use / extern crate. Target = what is imported, fully qualified (module or item).
    Depends(QName),
    /// Item defined at module level.
    Symbol(String, Visibility),
    /// Path call; the callee is resolved through the file's imports when possible.
    Call(QName),
    /// Type named in a signature, field, annotation.
    TypeRef(QName),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fact {
    pub at: Location,
    pub module: QName,
    pub kind: Kind,
}

#[derive(Default)]
pub struct Extraction {
    pub facts: Vec<Fact>,
    /// Files the adapter handles but could not parse cleanly (syntax error at that location).
    pub invalid: Vec<Location>,
}

/// The port. Adding a language = one more implementation, registered in the `Registry`.
pub trait Extractor: Sync {
    fn language(&self) -> Lang;
    /// Separator of the language's native notation, used to read and print qualified names.
    fn separator(&self) -> &'static str;
    /// Source files this adapter extracts from.
    fn handles(&self, path: &str) -> bool;
    /// Whole snapshot in, facts out: module identity needs layout files (`Cargo.toml`, packages).
    fn extract(&self, snapshot: &[SourceFile]) -> Extraction;
}

pub struct Registry {
    extractors: Vec<Box<dyn Extractor>>,
}

pub struct Report {
    pub facts: Vec<Fact>,
    pub invalid: Vec<Location>,
    /// Files no extractor handles: "not verified".
    pub unchecked: Vec<String>,
}

impl Registry {
    pub fn new(extractors: Vec<Box<dyn Extractor>>) -> Self {
        Registry { extractors }
    }

    fn get(&self, lang: &str) -> Option<&dyn Extractor> {
        self.extractors.iter().map(|e| &**e).find(|e| e.language() == lang)
    }

    /// Deterministic: output sorted, independent of file order and adapter internals.
    pub fn extract(&self, snapshot: &[SourceFile]) -> Report {
        let mut facts = Vec::new();
        let mut invalid = Vec::new();
        for e in &self.extractors {
            let x = e.extract(snapshot);
            facts.extend(x.facts);
            invalid.extend(x.invalid);
        }
        facts.sort();
        facts.dedup();
        invalid.sort();
        let mut unchecked: Vec<String> = snapshot
            .iter()
            .filter(|f| !self.extractors.iter().any(|e| e.handles(&f.path)))
            .map(|f| f.path.clone())
            .collect();
        unchecked.sort();
        Report { facts, invalid, unchecked }
    }

    /// `"rust:arbitre_core::store"` -> QName, through the adapter's separator.
    pub fn parse_name(&self, text: &str) -> Result<QName, String> {
        let (lang, path) = text.split_once(':').ok_or(format!("missing language in {text:?}"))?;
        let e = self.get(lang).ok_or(format!("language not supported: {lang}"))?;
        let segments = path.trim().split(e.separator()).map(str::to_owned).collect();
        Ok(QName { lang: e.language(), segments })
    }

    pub fn render(&self, q: &QName) -> String {
        let sep = self.get(q.lang).map_or("/", |e| e.separator());
        format!("{}:{}", q.lang, q.segments.join(sep))
    }
}

/// Toy rule: "modules under `from` must not depend on anything under `to`".
pub struct Forbid {
    pub from: QName,
    pub to: QName,
}

impl Forbid {
    /// `"rust:a::b !-> rust:c"`.
    pub fn parse(line: &str, reg: &Registry) -> Result<Self, String> {
        let (from, to) = line.split_once("!->").ok_or(format!("expected `!->` in {line:?}"))?;
        Ok(Forbid { from: reg.parse_name(from.trim())?, to: reg.parse_name(to.trim())? })
    }
}

/// Pure; facts are sorted, so violations come out sorted by location.
pub fn check<'a>(facts: &'a [Fact], rules: &[Forbid]) -> Vec<(usize, &'a Fact)> {
    let mut out = Vec::new();
    for f in facts {
        if let Kind::Depends(target) = &f.kind {
            for (i, r) in rules.iter().enumerate() {
                if f.module.starts_with(&r.from) && target.starts_with(&r.to) {
                    out.push((i, f));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake;
    impl Extractor for Fake {
        fn language(&self) -> Lang {
            "fake"
        }
        fn separator(&self) -> &'static str {
            "/"
        }
        fn handles(&self, path: &str) -> bool {
            path.ends_with(".fk")
        }
        // One fact per line `dep X`, module = file stem.
        fn extract(&self, snapshot: &[SourceFile]) -> Extraction {
            let mut x = Extraction::default();
            for f in snapshot.iter().filter(|f| self.handles(&f.path)) {
                let module = QName { lang: "fake", segments: vec![f.path.trim_end_matches(".fk").into()] };
                for (n, l) in f.text.lines().enumerate() {
                    if let Some(t) = l.strip_prefix("dep ") {
                        let at = Location { path: f.path.clone(), line: n as u32 + 1, column: 1, byte: 0 };
                        let target = QName { lang: "fake", segments: t.split('/').map(Into::into).collect() };
                        x.facts.push(Fact { at, module: module.clone(), kind: Kind::Depends(target) });
                    }
                }
            }
            x
        }
    }

    fn file(p: &str, t: &str) -> SourceFile {
        SourceFile { path: p.into(), text: t.into() }
    }

    #[test]
    fn rule_prefix_and_determinism() {
        let reg = Registry::new(vec![Box::new(Fake)]);
        let rules = [Forbid::parse("fake:core !-> fake:db", &reg).unwrap()];
        let a = vec![file("core.fk", "dep db/conn\ndep dbx\n"), file("app.fk", "dep db\n"), file("README", "")];
        let mut b: Vec<SourceFile> = a.iter().rev().map(|f| file(&f.path, &f.text)).collect();
        b.swap(0, 1);
        let (ra, rb) = (reg.extract(&a), reg.extract(&b));
        assert_eq!(ra.facts, rb.facts);
        let v = check(&ra.facts, &rules);
        assert_eq!(v.len(), 1);
        assert_eq!((v[0].1.at.path.as_str(), v[0].1.at.line), ("core.fk", 1));
        assert_eq!(ra.unchecked, vec!["README".to_string()]);
        assert!(Forbid::parse("cobol:a !-> cobol:b", &reg).is_err());
    }
}
