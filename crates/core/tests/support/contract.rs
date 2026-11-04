//! Contract suite of the `Store` port and its reference model, generic over the adapter
//! (PROP-ST02-9; plan T-6). Included by `tests/store.rs` and `crates/sqlite/tests/contract.rs`.
#![allow(dead_code)]

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use arbitre_core::*;

/// An adapter under test; owns its handle (and, on SQLite, its path).
pub trait Harness {
    type Store: Store;
    fn fresh() -> Self;
    fn store(&self) -> &Self::Store;
    /// Drops the handle and opens the same store again; panics on failure.
    fn reopen(&mut self);
}

/// Polls `f` once with a no-op waker; a store future must be ready (RG-ST02-6.3).
pub fn ready<F: Future>(f: F) -> F::Output {
    match pin!(f).poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("store future not ready on first poll (RG-ST02-6.3)"),
    }
}

/// Runs the model-based suite on `H` (1,000 sequences, `support::config()`).
pub fn run<H: Harness>() {
    todo!("T-6: contract suite — PROP-ST02-9")
}
