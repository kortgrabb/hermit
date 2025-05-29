use anyhow::Result;
use reedline::{DefaultPrompt, FileBackedHistory, Reedline, Signal};

use crate::alias::resolve_aliases;
use crate::executor::execute_command;
use crate::helpers::{LogLevel, log};
use crate::parser::parse_line;
use crate::shell::ShellInstance;
use crate::shellcommand::ShellCommand;

/// Processes a single command line
pub fn process_line(buffer: String, shell_instance: &mut ShellInstance) -> Result<()> {
    let line = buffer.trim();
    if line.is_empty() {
        return Ok(());
    }

    let mut tokens = parse_line(line, &shell_instance.env_vars);

    if tokens.is_empty() {
        return Ok(()); // Skip empty commands after expansion
    }

    // Resolve aliases before parsing command
    resolve_aliases(&mut tokens, &shell_instance.aliases);

    let command: ShellCommand = ShellCommand::from_tokens(&tokens)?;
    execute_command(command, &tokens, shell_instance);
    Ok(())
}

/// Runs the main REPL (Read-Eval-Print Loop)
pub fn run_repl() -> Result<()> {
    let mut shell_instance = ShellInstance::new();

    let hist = Box::new(
        FileBackedHistory::with_file(100, "history.txt".into())
            .expect("error creating history file"),
    );

    // Create default prompt
    let mut rl = Reedline::create().with_history(hist);
    let prompt = DefaultPrompt::default();

    loop {
        match rl.read_line(&prompt)? {
            Signal::Success(buffer) => {
                if let Err(e) = process_line(buffer, &mut shell_instance) {
                    log(&format!("Error processing command: {e}"), LogLevel::Error);
                }
            }
            Signal::CtrlC | Signal::CtrlD => {
                println!("cya!");
                break;
            }
        }
    }

    Ok(())
}
