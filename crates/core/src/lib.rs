//! Core of arbitre: stamps, hybrid clock, writes, merge.
//! Design: `.ctdd/stories/ST-01/03_archi.md` and amendments A-1 to A-11 in `04_plan.md` (French history).
//! `no_std` + `alloc`: the compiler forbids system time and I/O (A-6).

#![no_std]
// `vec!` for cargo-mutants "empty function" mutants (07_review R-1).
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

mod catalog;
mod clock;
mod error;
mod hlc;
mod merge;
#[cfg(feature = "sim")]
mod sim_store;
mod store;
pub mod value;
mod write;

pub use catalog::*;
pub use clock::*;
pub use error::*;
pub use hlc::*;
pub use merge::*;
#[cfg(feature = "sim")]
pub use sim_store::*;
pub use store::*;
// `value::Value` stays under its module: the root `Value` is `write::Value` (ST-03 skeleton).
pub use value::{Output, encode, render};
pub use write::*;
