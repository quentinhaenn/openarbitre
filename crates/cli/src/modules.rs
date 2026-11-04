//! Wired modules: one `mod` line and one list line per module (ST-03 plan P-4, P-5).
//! Provisional execution seam; ST-04 replaces it with `Module` and `Context` in core.

use arbitre_core::value::Value;
use arbitre_core::{ActionDescriptor, ActionError, Author, Catalog, CatalogError};
use arbitre_sqlite::SqliteStore;

#[cfg(feature = "test-actions")]
#[path = "wiring/test.rs"]
mod test;

/// What an action runs with (provisional until ST-04).
pub struct Context<'a> {
    pub store: &'a SqliteStore,
    /// Physical time, ms since the Unix epoch.
    pub pt: u64,
    pub author: Author,
}

/// One module: its id, its descriptors, its execution entry (conforming input in).
pub struct Wiring {
    pub id: &'static str,
    pub descriptors: fn() -> Vec<ActionDescriptor>,
    pub execute: fn(&str, Value, &Context<'_>) -> Result<Value, ActionError>,
}

/// Shipped modules (none in ST-03).
fn shipped() -> Vec<Wiring> {
    Vec::new()
}

/// Shipped modules, plus the test module with `test-actions`.
pub fn all() -> Vec<Wiring> {
    #[allow(unused_mut)]
    let mut all = shipped();
    #[cfg(feature = "test-actions")]
    all.extend(test::wirings());
    all
}

/// Builds the catalogue from `(id, descriptors())` of each wiring (RG-ST03-4.6).
pub fn catalog(wirings: &[Wiring]) -> Result<Catalog, CatalogError> {
    let modules: Vec<_> = wirings.iter().map(|w| (w.id, (w.descriptors)())).collect();
    Catalog::build(&modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_catalogue_builds() {
        // E-ST03-7 "without test-actions" (DA F-7).
        catalog(&shipped()).expect("shipped catalogue valid");
    }

    #[test]
    fn full_catalogue_builds() {
        let c = catalog(&all()).expect("catalogue valid");
        #[cfg(feature = "test-actions")]
        assert!(c.actions().iter().any(|a| a.name == "test.echo"));
        let _ = c;
    }
}
