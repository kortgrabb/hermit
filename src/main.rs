use std::{
    collections::HashMap,
    env,
    process::{self, Command, Stdio},
};

use anyhow::Result;
use color::Colors;
use helpers::LogLevel;
use reedline::{DefaultPrompt, FileBackedHistory, Reedline, Signal};
use shellcommand::{BuiltinCommand, ShellCommand};
use shlex::Shlex;

mod color;
mod helpers;
mod shellcommand;

fn execute_command(command: ShellCommand, tokens: &[String], variables: &HashMap<&str, &str>) {
    match command {
        ShellCommand::Builtin(builtin) => run_builtin(builtin, variables),
        ShellCommand::External() => run_external(&tokens),
    }
}

fn run_external(tokens: &[String]) {
    let cmd = &tokens[0];
    // Spawn a new process with inheritance of stdin, stdout, and stderr
    match Command::new(cmd)
        .args(&tokens[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(mut child) => {
            if let Err(e) = child.wait() {
                eprintln!("shell: failed to wait on child: {e}");
            }
        }
        Err(e) => {
            // TODO: replace hermit with dynamic name
            helpers::log(format!("hermit: {cmd}: {e}").as_str(), LogLevel::Error);
        }
    }
}

fn run_builtin(command_builtin: BuiltinCommand, variables: &HashMap<&str, &str>) {
    match command_builtin {
        BuiltinCommand::Exit => process::exit(0),
        BuiltinCommand::Cd(target) => {
            if let Err(e) = env::set_current_dir(&target) {
                helpers::log(format!("cd: {e}").as_str(), LogLevel::Error);
            }
        }
        BuiltinCommand::Echo(message) => {
            println!("{message}")
        }
        _ => {}
    }
}

fn main() -> Result<()> {
    let hist = Box::new(
        FileBackedHistory::with_file(100, "history.txt".into())
            .expect("error creating history file"),
    );

    // create default prompt
    let mut rl = Reedline::create().with_history(hist);
    let prompt = DefaultPrompt::default();

    let env_vars: HashMap<&str, &str> = HashMap::with_capacity(100);

    loop {
        match rl.read_line(&prompt)? {
            Signal::Success(buffer) => {
                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }

                // Tokenise with POSIX-like quoting support
                let lexer = Shlex::new(line);
                let mut tokens: Vec<String> = lexer.collect();
                if tokens.is_empty() {
                    continue;
                }

                for (k, v) in &env_vars {
                    if &tokens[0].as_str() == k {
                        tokens[0] = v.to_string();
                    }
                }

                let command: ShellCommand = ShellCommand::from_tokens(&tokens)?;
                execute_command(command, &tokens, &env_vars);
            }
            Signal::CtrlC | Signal::CtrlD => {
                println!("cya!");
                break;
            }
        }
    }

    Ok(())
}
