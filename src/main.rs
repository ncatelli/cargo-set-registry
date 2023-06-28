use clap::Parser;
use std::process;

mod cli;
mod errors;

fn main() {
    let args = cli::Command::parse();

    if let Err(err) = args.exec() {
        eprintln!("Error: {err:?}");

        process::exit(1);
    }
}
