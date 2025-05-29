use std::collections::HashMap;

/// Resolves aliases in the given tokens, preventing infinite recursion
pub fn resolve_aliases(tokens: &mut Vec<String>, aliases: &HashMap<String, String>) {
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
