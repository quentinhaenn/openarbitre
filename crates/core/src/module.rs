//! Module contract, action context, namespace-bound store view, pre-parse helpers
//! (ST-04 specs RG-ST04-1.5, 1.6, 2.2, 3.1 to 3.3, 7.1; plan P-1 to P-6).

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;

use crate::ai::Ai;
use crate::catalog::ActionDescriptor;
use crate::clock::Clock;
use crate::error::ActionError;
use crate::hlc::DeviceId;
use crate::merge::RecordState;
use crate::store::{ApplyError, Store, StoreError};
use crate::value::{Output, Value};
use crate::write::{Author, Change, RecordId, Write};

/// Future returned by `Module` and `Ai` methods (dyn-compatible, Q3 (a)).
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// One finding of `Module::revalidate` (RG-ST04-7.1, frozen).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub record: RecordId,
    /// Stable `snake_case` id.
    pub id: String,
    pub message: String,
}

/// A toggleable module (RG-ST04-3.1, frozen). `S` is the store adapter.
pub trait Module<S: Store> {
    /// First segment of its action names; also its namespace.
    fn id(&self) -> &'static str;

    fn actions(&self) -> Vec<ActionDescriptor>;

    /// Runs `action` on a conforming `input` (RG-ST03-4.4); the output is checked after.
    fn execute<'a>(
        &'a self,
        action: &'a str,
        input: Value,
        ctx: &'a Context<'a, S>,
    ) -> LocalBoxFuture<'a, Result<Output, ActionError>>;

    /// Re-checks `records` (`(category, id)` of its namespace); reads only (CONFLICT-1).
    fn revalidate<'a>(
        &'a self,
        records: &'a [(String, String)],
        view: &'a ModuleStore<'a, S>,
    ) -> LocalBoxFuture<'a, Result<Vec<Problem>, StoreError>>;
}

/// What an action runs with, built by the surface (RG-ST04-3.2, frozen).
pub struct Context<'a, S> {
    pub author: Author,
    pub clock: &'a dyn Clock,
    pub store: ModuleStore<'a, S>,
    /// `None` on every surface until ST-30.
    pub ai: Option<&'a dyn Ai>,
}

/// The store as module `module` sees it: no method names a module (RG-ST04-3.3, frozen).
///
/// Fields are private, so a module cannot reach the whole store (DA F-4):
///
/// ```compile_fail
/// fn leak<'a, S: arbitre_core::Store>(v: &arbitre_core::ModuleStore<'a, S>) -> &'a S {
///     v.store
/// }
/// ```
pub struct ModuleStore<'a, S> {
    store: &'a S,
    module: &'static str,
    author: Author,
    clock: &'a dyn Clock,
}

impl<'a, S: Store> ModuleStore<'a, S> {
    pub fn new(store: &'a S, module: &'static str, author: Author, clock: &'a dyn Clock) -> Self {
        Self {
            store,
            module,
            author,
            clock,
        }
    }

    /// Namespace of this view.
    pub fn module(&self) -> &'static str {
        self.module
    }

    pub async fn device(&self) -> Result<DeviceId, StoreError> {
        todo!("T-2 {:p}", self.store)
    }

    /// Projected state of `(module, category, id)`.
    pub async fn state(&self, category: &str, id: &str) -> Result<Option<RecordState>, StoreError> {
        todo!("T-2 {category} {id} {:?}", (self.module, self.author))
    }

    /// Records of a category of this namespace, sorted by id: `(id, state)`.
    pub async fn records(&self, category: &str) -> Result<Vec<(String, RecordState)>, StoreError> {
        todo!("T-2 {category}")
    }

    /// Writes of `(module, category, id)`, `Log::writes` order.
    pub async fn writes_of(&self, category: &str, id: &str) -> Result<Vec<Write>, StoreError> {
        todo!("T-2 {category} {id}")
    }

    /// One atomic local apply of `(category, id, change)`, stamped with the view's clock
    /// and author.
    pub async fn apply(
        &self,
        changes: &[(String, String, Change)],
    ) -> Result<Vec<Write>, ApplyError> {
        todo!("T-2 {changes:?} {}", self.clock.now())
    }
}

/// What the binary needs before parsing (RG-ST04-2.2, 2.3; plan P-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PreScan<'a> {
    /// Last `--store x` or `--store=x` before `--`.
    pub store: Option<&'a str>,
    /// First argument that is neither an option nor `--store`'s value, before `--`.
    pub first_word: Option<&'a str>,
}

/// Reads `args` (program name excluded) without the catalogue.
pub fn pre_scan<'a>(args: &[&'a str]) -> PreScan<'a> {
    todo!("T-2 {args:?}")
}

/// Record holding the enabled state of `device`: `arbitre/module-state/<device hex>` (P-6).
pub fn state_record(device: DeviceId) -> RecordId {
    todo!("T-2 {device:?}")
}

/// Enabled modules among `build`: a field `b"1"` enables, anything else gives the default
/// (off); fields naming ids outside `build` are ignored (RG-ST04-1.6, DA F-15).
pub fn enabled_from(state: Option<&RecordState>, build: &[&'static str]) -> BTreeSet<&'static str> {
    todo!("T-2 {state:?} {build:?}")
}

/// Change writing `module` on (`b"1"`) or off (`b"0"`) in the state record.
pub fn state_change(module: &str, on: bool) -> Change {
    todo!("T-2 {module} {on}")
}
