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

struct ShellInstance {
    env_vars: HashMap<String, String>,
    aliases: HashMap<String, String>,
}

fn execute_command(command: ShellCommand, tokens: &[String], shell_instance: &mut ShellInstance) {
    match command {
        ShellCommand::Builtin(builtin) => run_builtin(
            builtin,
            &mut shell_instance.env_vars,
            &mut shell_instance.aliases,
        ),
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

fn run_builtin(
    command_builtin: BuiltinCommand,
    env_vars: &mut HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) {
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
        BuiltinCommand::Alias(key, value) => {
            aliases.insert(key, value);
        }
        _ => {}
    }
}

fn resolve_aliases(tokens: &mut Vec<String>, aliases: &HashMap<String, String>) {
    if tokens.is_empty() {
        return;
    }

    let mut expanded_aliases = std::collections::HashSet::new();
    let mut resolved = false;

    while !resolved {
        let current_command = &tokens[0];

        // Check if we've already expanded this alias to prevent infinite recursion
        if expanded_aliases.contains(current_command) {
            resolved = true;
            continue;
        }

        if let Some(alias_value) = aliases.get(current_command) {
            // Parse the alias value into tokens
            let alias_tokens: Vec<String> = alias_value
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            if !alias_tokens.is_empty() {
                // Mark this alias as expanded
                expanded_aliases.insert(current_command.clone());

                // Replace the first token with the alias expansion
                tokens.remove(0);
                for (i, token) in alias_tokens.into_iter().enumerate() {
                    tokens.insert(i, token);
                }
            } else {
                resolved = true;
            }
        } else {
            resolved = true;
        }
    }
}

fn process_line(buffer: String, shell_instance: &mut ShellInstance) -> Result<()> {
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
            let last = chars.last();

            match (first, last) {
                (Some('\''), Some('\'')) if s.len() >= 2 => s[1..s.len() - 1].to_string(),
                (Some('"'), Some('"')) if s.len() >= 2 => s[1..s.len() - 1].to_string(),
                _ => s.to_string(),
            }
        })
        .collect();

    tokens = expand_env_vars(trimmed_tokens, &shell_instance.env_vars);

    if tokens.is_empty() {
        return Ok(()); // Skip empty commands after expansion
    }

    // Resolve aliases before parsing command
    resolve_aliases(&mut tokens, &shell_instance.aliases);

    let command: ShellCommand = ShellCommand::from_tokens(&tokens)?;
    execute_command(command, &tokens, shell_instance);
    Ok(())
}

fn run_repl() -> Result<()> {
    let mut shell_instance = ShellInstance {
        env_vars: HashMap::new(),
        aliases: HashMap::new(),
    };

    for env_var in env::vars() {
        shell_instance.env_vars.insert(env_var.0, env_var.1);
    }

    let hist = Box::new(
        FileBackedHistory::with_file(100, "history.txt".into())
            .expect("error creating history file"),
    );

    // create default prompt
    let mut rl = Reedline::create().with_history(hist);
    let prompt = DefaultPrompt::default();

    loop {
        match rl.read_line(&prompt)? {
            Signal::Success(buffer) => {
                if let Err(e) = process_line(buffer, &mut shell_instance) {
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

fn main() -> Result<()> {
    if let Err(e) = run_repl() {
        eprintln!("Error running REPL: {e}");
        process::exit(1);
    }
    Ok(())
}
