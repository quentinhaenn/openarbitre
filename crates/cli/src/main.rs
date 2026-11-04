//! `arbitre` binary: catalogue → derived command line → store → action → one result
//! (ST-03 specs RG-ST03-2.1, 2.2, 3.5; plan T-6).
#![expect(dead_code, reason = "ST-03 skeleton: wired by T-6")]

mod cli;
mod modules;
mod store;

/// `--json` among the raw arguments, before any `--` (RG-ST03-2.1).
fn json_mode(args: &[std::ffi::OsString]) -> bool {
    let _ = args;
    todo!("T-6: json_mode — RG-ST03-2.1")
}

fn main() {
    todo!("T-6: main pipeline — RG-ST03-2.1, 2.2, 3.5, plan F-5")
}
