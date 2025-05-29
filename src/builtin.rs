use crate::helpers::{LogLevel, log};
use crate::shellcommand::BuiltinCommand;
use std::{collections::HashMap, env, process};

/// Executes a builtin command
pub fn run_builtin(
    command_builtin: BuiltinCommand,
    env_vars: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) {
    match command_builtin {
        BuiltinCommand::Exit => process::exit(0),
        BuiltinCommand::Cd(target) => {
            if let Err(e) = env::set_current_dir(&target) {
                log(&format!("cd: {e}"), LogLevel::Error);
            }
        }
        BuiltinCommand::Echo(message) => {
            println!("{message}")
        }
        BuiltinCommand::Set(var, value) => {
            env_vars.insert(var, value);
        }
        BuiltinCommand::Unset(var) => {
            env_vars.remove(&var);
        }
        BuiltinCommand::Env() => {
            for (key, value) in env_vars.iter() {
                println!("{}={}", key, value);
            }
        }
        BuiltinCommand::Alias(key, value) => {
            aliases.insert(key, value);
        }
        BuiltinCommand::Help => {
            print_help();
        }
        BuiltinCommand::History => {
            // TODO: Implement history display
            println!("History functionality not yet implemented");
        }
    }
}

/// Prints help information for the shell
fn print_help() {
    println!("HERMIT Shell - Available Commands:");
    println!("  exit        - Exit the shell");
    println!("  cd <dir>    - Change directory");
    println!("  echo <msg>  - Print message");
    println!("  set <var> <value> - Set environment variable");
    println!("  unset <var> - Unset environment variable");
    println!("  env         - Show all environment variables");
    println!("  alias <name> <command> - Create an alias");
    println!("  help        - Show this help message");
    println!("  history     - Show command history");
}
