use std::{
    env,
    process::{self, Command, Stdio},
};

use anyhow::Result;
use color::Colors;
use reedline::{DefaultPrompt, FileBackedHistory, Reedline, Signal};
use shellcommand::{BuiltinCommand, ShellCommand};
use shlex::Shlex;

mod color;
mod shellcommand;

fn run_external(tokens: &[String]) {
    let cmd = &tokens[0];
    match Command::new(cmd) // fork the called command
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
            eprintln!("shell: {cmd}: {e}");
        }
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

    loop {
        match rl.read_line(&prompt)? {
            Signal::Success(buffer) => {
                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }

                // Tokenise with POSIX-like quoting support
                let lexer = Shlex::new(line);
                let tokens: Vec<String> = lexer.collect();
                if tokens.is_empty() {
                    continue;
                }

                let command: ShellCommand = ShellCommand::from_tokens(&tokens)?;
                execute_command(command, &tokens);

                // NOTE: Temporary
                println!("[TOKENS]");
                for t in tokens {
                    println!("{t}");
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

fn execute_command(command: ShellCommand, tokens: &[String]) {
    match command {
        ShellCommand::Builtin(builtin) => match builtin {
            BuiltinCommand::Exit => process::exit(0),
            BuiltinCommand::Cd(target) => {
                // TODO: Custom print handling
                if let Err(e) = env::set_current_dir(&target) {
                    eprintln!("{}cd: {e}", Colors::Red.to_string());
                }
            }
            BuiltinCommand::Echo(message) => {
                println!("{message}");
            }
            _ => {}
        },
        ShellCommand::External() => run_external(&tokens),
    }
}
