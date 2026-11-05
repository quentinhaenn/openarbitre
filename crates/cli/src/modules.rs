//! Toggleable modules: one `mod` line and one list line per module (ST-04 plan P-9).
//! The foundation (`arbitre`) is not listed here: `wiring/arbitre.rs`.

use arbitre_core::{Catalog, CatalogError, Module};
use arbitre_sqlite::SqliteStore;

#[path = "wiring/arbitre.rs"]
pub mod foundation;

#[cfg(feature = "test-actions")]
#[path = "wiring/fake.rs"]
mod fake;

pub type Modules = Vec<Box<dyn Module<SqliteStore>>>;

/// Toggleable modules of this build (none shipped before ST-05).
pub fn all() -> Modules {
    #[allow(unused_mut)]
    let mut all: Modules = Vec::new();
    #[cfg(feature = "test-actions")]
    all.extend(fake::modules());
    all
}

/// Ids of `modules`, in list order.
pub fn ids(modules: &Modules) -> Vec<&'static str> {
    modules.iter().map(|m| m.id()).collect()
}

/// Catalogue of the foundation and the `modules` kept by `enabled` (RG-ST03-4.6,
/// RG-ST04-2.1, 3.4); the `module` enum always names the whole build.
pub fn catalog(modules: &Modules, enabled: impl Fn(&str) -> bool) -> Result<Catalog, CatalogError> {
    let listed: Vec<_> = modules
        .iter()
        .filter(|m| enabled(m.id()))
        .map(|m| (m.id(), m.actions()))
        .collect();
    Catalog::build(&foundation::descriptors(&ids(modules)), &listed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_catalogue_builds() {
        // E-ST03-7 "without test-actions" (DA F-7 of ST-03 and of ST-04).
        let c = catalog(&Vec::new(), |_| true).expect("shipped catalogue valid");
        let names: Vec<_> = c.actions().iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"arbitre.module.list"));
        assert!(!names.contains(&"arbitre.module.enable"));
    }

    #[test]
    fn full_catalogue_builds() {
        let c = catalog(&all(), |_| true).expect("catalogue valid");
        let test_actions = cfg!(feature = "test-actions");
        let has = |n: &str| c.actions().iter().any(|a| a.name == n);
        assert!(!test_actions || (has("arbitre.test.echo") && has("fake-a.put")));
    }
}
