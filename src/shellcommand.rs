use anyhow::{Ok, Result, anyhow};

pub enum ShellCommand {
    Builtin(BuiltinCommand),
    External(),
}
pub enum BuiltinCommand {
    Help,
    Exit,
    Cd(String),
    Echo(String),
    History,
    Set(String, String),
}

impl ShellCommand {
    pub fn from_tokens(tokens: &[String]) -> Result<Self> {
        if tokens.is_empty() {
            return Err(anyhow!("No command provided"));
        }

        match tokens[0].as_str() {
            "exit" => Ok(ShellCommand::Builtin(BuiltinCommand::Exit)),
            "cd" => {
                let target = if tokens.len() > 1 { &tokens[1] } else { "." };
                Ok(ShellCommand::Builtin(BuiltinCommand::Cd(
                    target.to_string(),
                )))
            }
            "echo" => {
                let message = tokens[1..].join(" ");
                Ok(ShellCommand::Builtin(BuiltinCommand::Echo(message)))
            }
            "history" => Ok(ShellCommand::Builtin(BuiltinCommand::History)),
            "help" => Ok(ShellCommand::Builtin(BuiltinCommand::Help)),
            "set" => {
                if tokens.len() < 3 {
                    return Err(anyhow!("Missing arguments for 'set' command"));
                }
                let alias = tokens[1].clone();
                let real = tokens[2].clone();
                Ok(ShellCommand::Builtin(BuiltinCommand::Set(alias, real)))
            }
            _ => Ok(ShellCommand::External()),
        }
    }
}
