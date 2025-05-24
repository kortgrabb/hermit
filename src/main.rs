use std::{
    collections::HashMap,
    env,
    process::{self, Command, Stdio},
};

use anyhow::Result;
use helpers::LogLevel;
use reedline::{DefaultPrompt, FileBackedHistory, Reedline, Signal};
use shellcommand::{BuiltinCommand, ShellCommand};
use shlex::Shlex;

mod color;
mod helpers;
mod shellcommand;

fn execute_command(
    command: ShellCommand,
    tokens: &[String],
    env_vars: &mut HashMap<String, String>,
) {
    match command {
        ShellCommand::Builtin(builtin) => run_builtin(builtin, env_vars),
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
            helpers::log(format!("shell: {cmd}: {e}").as_str(), LogLevel::Error);
        }
    }
}

fn run_builtin(command_builtin: BuiltinCommand, env_vars: &mut HashMap<String, String>) {
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
        BuiltinCommand::Set(var, value) => {
            env_vars.insert(var, value);
        }
        _ => {}
    }
}

fn expand_env_vars(tokens: Vec<String>, env_vars: &HashMap<String, String>) -> Result<Vec<String>> {
    let mut expanded_tokens = Vec::new();
    for token in tokens {
        if token.starts_with('$') {
            let var_name = &token[1..]; // Remove the leading '$'
            if let Some(value) = env_vars.get(var_name) {
                expanded_tokens.push(value.clone());
            } else {
                helpers::log(
                    format!("Warning: Environment variable '{}' not found", var_name).as_str(),
                    LogLevel::Warning,
                );
            }
        } else {
            expanded_tokens.push(token);
        }
    }
    Ok(expanded_tokens)
}

fn main() -> Result<()> {
    let hist = Box::new(
        FileBackedHistory::with_file(100, "history.txt".into())
            .expect("error creating history file"),
    );

    // create default prompt
    let mut rl = Reedline::create().with_history(hist);
    let prompt = DefaultPrompt::default();

    let mut env_vars: HashMap<String, String> = HashMap::with_capacity(100);

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

                // Check for environment variable expansion
                tokens = expand_env_vars(tokens, &env_vars)?;

                let command: ShellCommand = ShellCommand::from_tokens(&tokens)?;
                execute_command(command, &tokens, &mut env_vars);
            }
            Signal::CtrlC | Signal::CtrlD => {
                println!("cya!");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_external() {
        let tokens = vec!["echo".to_string(), "Hello, World!".to_string()];
        run_external(&tokens);
    }

    #[test]
    fn test_expand_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("USER".to_string(), "testuser".to_string());
        let tokens = vec!["Hello".to_string(), "$USER".to_string()];
        let expanded = expand_env_vars(tokens, &env_vars).unwrap();
        assert_eq!(expanded, vec!["Hello".to_string(), "testuser".to_string()]);
    }
}
