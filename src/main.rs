use std::{
    env,
    process::{Command, Stdio},
};

use anyhow::Result;
use crossterm::style::Color;
use reedline::{DefaultPrompt, FileBackedHistory, Prompt, Reedline, Signal};
use shlex::Shlex;

enum Colors {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
impl Colors {
    fn to_string(&self) -> String {
        match self {
            Colors::Red => "\x1b[31m",
            Colors::Green => "\x1b[32m",
            Colors::Yellow => "\x1b[33m",
            Colors::Blue => "\x1b[34m",
            Colors::Magenta => "\x1b[35m",
            Colors::Cyan => "\x1b[36m",
            Colors::White => "\x1b[37m",
        }
        .to_string()
    }
}

fn run_builtin(tokens: &[String]) -> bool {
    match tokens[0].as_str() {
        "exit" => {
            // graceful shutdown handled by caller
            return false;
        }
        "cd" => {
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let target = if tokens.len() > 1 { &tokens[1] } else { &home };
            if let Err(e) = env::set_current_dir(&target) {
                eprintln!("cd: {e}");
            }
        }
        _ => return true, // not a builtin
    }
    true
}

fn run_external(tokens: &[String]) {
    let cmd = &tokens[0];
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
            eprintln!("shell: {cmd}: {e}");
        }
    }
}

struct ShellPrompt;

impl Prompt for ShellPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        let current_dir = env::current_dir()
            .unwrap_or_else(|_| env::current_exe().unwrap())
            .to_string_lossy()
            .to_string();

        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path_pretty = if current_dir.starts_with(&home) {
            current_dir.replacen(&home, "~", 1)
        } else {
            current_dir
        };

        // final output: ~/path/to/dir $ (in blue)
        let prompt = format!("{path_pretty}{} $ ", Colors::Blue.to_string());

        prompt.into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        let host = env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
        let user = env::var("USER").unwrap_or_else(|_| "user".to_string());

        let merged = format!("{}@{} ", user, host);
        let color = Colors::Cyan.to_string();
        let merged = format!("{}{}", color, merged);

        merged.into()
    }

    fn render_prompt_indicator(
        &self,
        prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        let color = Colors::Cyan.to_string();
        format!("{}| ", color).into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        "".into()
    }
}

fn main() -> Result<()> {
    let hist = Box::new(
        FileBackedHistory::with_file(100, "history.txt".into())
            .expect("error creating history file"),
    );
    let mut rl = Reedline::create().with_history(hist);
    let prompt = ShellPrompt;

    loop {
        match rl.read_line(&prompt)? {
            Signal::Success(buffer) => {
                let line = buffer.trim();
                if line.is_empty() {
                    continue;
                }

                // Tokenise with POSIX‑like quoting support
                let lexer = Shlex::new(line);
                let tokens: Vec<String> = lexer.collect();
                if tokens.is_empty() {
                    continue;
                }

                if !run_builtin(&tokens) {
                    break; // "exit" builtin
                }

                // If not a builtin, try external command
                if tokens[0] != "cd" && tokens[0] != "exit" {
                    run_external(&tokens);
                }
            }
            Signal::CtrlC | Signal::CtrlD => {
                println!();
                break;
            }
        }
    }

    Ok(())
}
