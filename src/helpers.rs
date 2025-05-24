use std::collections::HashMap;

use anyhow::Result;

use crate::color::Colors;

pub enum LogLevel {
    Normal,
    Warning,
    Error,
}

pub fn log(to_print: &str, level: LogLevel) {
    match level {
        LogLevel::Normal => {
            println!("[LOG]:\n{to_print}");
        }
        LogLevel::Warning => {
            println!("{}[WARN]:\n{to_print}{}", Colors::Yellow, Colors::Reset)
        }
        LogLevel::Error => {
            println!("{}[ERROR]:\n{to_print}{}", Colors::Red, Colors::Reset);
        }
    }
}

/// Check if a environment variable is valid.
pub fn is_valid_env_var(var: &str, env_vars: &HashMap<String, String>) -> bool {
    if let Some(value) = env_vars.get(var) {
        !value.is_empty()
    } else {
        false
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
            let var_name = &var[1..]; // Skip the '$' character
            if let Some(value) = env_vars.get(var_name) {
                // HACK: Use word boundaries to ensure exact variable name matching
                let pattern = format!("\\${}\\b", regex::escape(var_name));
                let re = regex::Regex::new(&pattern).unwrap();
                expanded_token = re.replace_all(&expanded_token, value).to_string();
            }
        }
        // Add the expanded token to the result
        expanded_tokens.push(expanded_token);
    }
    expanded_tokens
}
