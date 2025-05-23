use anyhow::{Result, anyhow};

pub enum ShellCommand {
    Builtin(BuiltinCommand),
    External(),
}

pub enum BuiltinCommand {
    Exit,
    Cd(String),
    Echo(String),
    History,
    Help,
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
            _ => Ok(ShellCommand::External()),
        }
    }
}
