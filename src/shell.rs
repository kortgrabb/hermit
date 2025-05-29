use std::collections::HashMap;

/// Represents the shell instance with its state
pub struct ShellInstance {
    pub env_vars: HashMap<String, String>,
    pub aliases: HashMap<String, String>,
}

impl ShellInstance {
    /// Creates a new shell instance with environment variables from the system
    pub fn new() -> Self {
        let mut shell_instance = Self {
            env_vars: HashMap::new(),
            aliases: HashMap::new(),
        };

        // Initialize with system environment variables
        for (key, value) in std::env::vars() {
            shell_instance.env_vars.insert(key, value);
        }

        shell_instance
    }
}

impl Default for ShellInstance {
    fn default() -> Self {
        Self::new()
    }
}
