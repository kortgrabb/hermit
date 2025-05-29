use crate::repl::run_repl;
use anyhow::Result;
use std::process;

mod alias;
mod builtin;
mod color;
mod executor;
mod helpers;
mod parser;
mod repl;
mod shell;
mod shellcommand;

fn main() -> Result<()> {
    if let Err(e) = run_repl() {
        eprintln!("Error running REPL: {e}");
        process::exit(1);
    }
    Ok(())
}
