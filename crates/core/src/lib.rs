//! Core of arbitre: stamps, hybrid clock, writes, merge.
//! Design: `.ctdd/stories/ST-01/03_archi.md` and amendments A-1 to A-11 in `04_plan.md` (French history).
//! `no_std` + `alloc`: the compiler forbids system time and I/O (A-6).

#![no_std]
// `vec!` for cargo-mutants "empty function" mutants (07_review R-1).
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

mod clock;
mod hlc;
mod merge;
mod write;

pub use clock::*;
pub use hlc::*;
pub use merge::*;
pub use write::*;
