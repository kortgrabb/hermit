use std::{
    collections::HashMap,
    env,
    process::{self, Command, Stdio},
};

use anyhow::Result;
use helpers::{LogLevel, expand_env_vars};
use reedline::{DefaultPrompt, FileBackedHistory, Reedline, Signal};
use shellcommand::{BuiltinCommand, ShellCommand};

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
        BuiltinCommand::Unset(var) => {
            env_vars.remove(&var);
        }
        BuiltinCommand::Env() => {
            for (key, value) in env_vars.iter() {
                println!("{}={}", key, value);
            }
        }
        _ => {}
    }
}

fn process_line(buffer: String, env_vars: &mut HashMap<String, String>) -> Result<()> {
    let line = buffer.trim();
    if line.is_empty() {
        return Ok(());
    }

    let mut tokens: Vec<String> = Vec::new();
    let mut current_token = String::new();
    let mut in_quote: Option<char> = None;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match in_quote {
            Some(quote_char) => {
                if ch == quote_char {
                    in_quote = None;
                    // Optionally, decide if quotes themselves should be part of the token
                    // or stripped here. Current logic strips them later.
                    current_token.push(ch);
                } else {
                    current_token.push(ch);
                }
            }
            None => {
                if ch == '\'' || ch == '"' {
                    in_quote = Some(ch);
                    // Optionally, decide if quotes themselves should be part of the token.
                    current_token.push(ch);
                } else if ch.is_whitespace() {
                    if !current_token.is_empty() {
                        tokens.push(current_token);
                        current_token = String::new();
                    }
                } else {
                    current_token.push(ch);
                }
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    // trim the quotes from the tokens if they are at the very start and end
    let trimmed_tokens: Vec<String> = tokens
        .iter()
        .map(|s| {
            let mut chars = s.chars();
            let first = chars.next();
            let last = chars.last(); // Consumes the iterator, so get first before last

            match (first, last) {
                (Some('\''), Some('\'')) if s.len() >= 2 => s[1..s.len() - 1].to_string(),
                (Some('"'), Some('"')) if s.len() >= 2 => s[1..s.len() - 1].to_string(),
                _ => s.to_string(),
            }
        })
        .collect();

    // Check for environment variable expansion
    tokens = expand_env_vars(trimmed_tokens, env_vars);

    if tokens.is_empty() {
        return Ok(()); // Skip empty commands after expansion
    }

    let command: ShellCommand = ShellCommand::from_tokens(&tokens)?;
    execute_command(command, &tokens, env_vars);
    Ok(())
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
                if let Err(e) = process_line(buffer, &mut env_vars) {
                    helpers::log(
                        format!("Error processing command: {e}").as_str(),
                        LogLevel::Error,
                    );
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
        let expanded = expand_env_vars(tokens, &env_vars);
        assert_eq!(expanded, vec!["Hello".to_string(), "testuser".to_string()]);
    }
}
