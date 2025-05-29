use crate::builtin::run_builtin;
use crate::helpers::{LogLevel, log};
use crate::shell::ShellInstance;
use crate::shellcommand::ShellCommand;
use std::process::{Command, Stdio};

/// Executes a command (either builtin or external)
pub fn execute_command(
    command: ShellCommand,
    tokens: &[String],
    shell_instance: &mut ShellInstance,
) {
    match command {
        ShellCommand::Builtin(builtin) => run_builtin(
            builtin,
            &mut shell_instance.env_vars,
            &mut shell_instance.aliases,
        ),
        ShellCommand::External() => run_external(tokens),
    }
}

/// Executes an external command
pub fn run_external(tokens: &[String]) {
    if tokens.is_empty() {
        log("No command to execute", LogLevel::Error);
        return;
    }

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
            log(&format!("shell: {cmd}: {e}"), LogLevel::Error);
        }
    }
}
