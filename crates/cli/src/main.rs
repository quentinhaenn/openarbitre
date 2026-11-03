//! Ligne de commande `arbitre` : parseur minimal en attendant ST-03.

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Arguments {}

fn main() {
    Arguments::parse();
}
