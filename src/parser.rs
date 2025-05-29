use crate::helpers::expand_env_vars;
use std::collections::HashMap;

/// Tokenizes a command line string into individual tokens, handling quotes properly
pub fn tokenize(line: &str) -> Vec<String> {
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

    tokens
}

/// Trims quotes from tokens if they are at the very start and end
pub fn trim_quotes(tokens: Vec<String>) -> Vec<String> {
    tokens
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
        .collect()
}

/// Parses a command line string into tokens, handling quotes and environment variables
pub fn parse_line(line: &str, env_vars: &HashMap<String, String>) -> Vec<String> {
    let tokens = tokenize(line);
    let trimmed_tokens = trim_quotes(tokens);
    expand_env_vars(trimmed_tokens, env_vars)
}
