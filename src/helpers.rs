use std::collections::HashMap;
use crate::color::Colors;

pub enum LogLevel {
    Warning,
    Error,
}

pub fn log(to_print: &str, level: LogLevel) {
    match level {
        LogLevel::Warning => {
            println!("{}[WARN]:\n{to_print}{}", Colors::Yellow, Colors::Reset)
        }
        LogLevel::Error => {
            println!("{}[ERROR]:\n{to_print}{}", Colors::Red, Colors::Reset);
        }
    }
}

pub fn expand_env_vars(tokens: Vec<String>, env_vars: &HashMap<String, String>) -> Vec<String> {
    let mut expanded_tokens = Vec::new();
    // IMPORTANT: We need to handle "single" tokens that may contain spaces
    // e.g., "echo '$USER is $USERNAME'" => ["echo", "$USER is $USERNAME"]
    for token in tokens {
        let variables = token
            .split_whitespace()
            .filter(|s| s.starts_with('$') && s.len() > 1)
            .collect::<Vec<&str>>();
        if variables.is_empty() {
            expanded_tokens.push(token);
            continue;
        }

        // Sort variables by length in descending order to replace longer variables first
        let mut sorted_variables = variables;
        sorted_variables.sort_by(|a, b| b.len().cmp(&a.len()));

        let mut expanded_token = token.clone();
        for var in sorted_variables {
            let var_name = &var[1..]; // Remove the leading '$'
            // get the substring up until a special character (e.g., !, @, #, etc.)
            let var_name = var_name
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();

            // only replace the var_name
            if let Some(value) = env_vars.get(&var_name) {
                // Replace the variable with its value
                expanded_token = expanded_token.replace(&format!("${}", var_name), value);
            } else {
                // If the variable is not found, log a warning
                log(
                    &format!("Environment variable '{}' not found", var_name),
                    LogLevel::Warning,
                );
            }
        }
        // Add the expanded token to the result
        expanded_tokens.push(expanded_token);
    }
    expanded_tokens
}
