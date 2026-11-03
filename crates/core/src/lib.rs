//! Cœur d'arbitre : marques, horloge hybride, écritures, fusion.
//! Architecture : `.ctdd/stories/ST-01/03_archi.md` et amendements A-1 à A-11 de `04_plan.md`.
//! `no_std` + `alloc` : le compilateur interdit l'heure système et les E/S (A-6).

#![no_std]
extern crate alloc;

mod clock;
mod hlc;
mod merge;
mod write;

pub use clock::*;
pub use hlc::*;
pub use merge::*;
pub use write::*;
