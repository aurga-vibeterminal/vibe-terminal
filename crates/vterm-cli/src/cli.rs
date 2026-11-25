use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "vtermc")]
#[command(about = "Console terminal client for vterm-server")]
#[command(version)]
pub struct Args {
    /// Select terminal profile (omit to list interactively)
    #[arg(short, long, value_name = "ID", num_args = 0..=1)]
    pub profile: Option<Option<String>>,

    /// Set working directory
    #[arg(short, long, value_name = "DIR")]
    pub working_dir: Option<String>,

    /// Select server and existing terminal interactively: clears the
    /// screen, lists servers from the client config (Local is always
    /// available), then lists that server's terminals to attach to.
    #[arg(short, long)]
    pub select: bool,

    /// Add a server to the client config. With a `vtshare1:` share string
    /// the name/address/secret key are decrypted from it; without a value
    /// they are prompted for interactively. Exits after saving.
    #[arg(short, long, value_name = "SHARE", num_args = 0..=1)]
    pub add: Option<Option<String>>,

    /// Server WebSocket URL
    #[arg(
        long,
        value_name = "URL",
        default_value = "wss://localhost:5153",
        env = "VTERM_SERVER"
    )]
    pub server: String,

    /// Additional environment variables (VAR=VALUE)
    #[arg(short, long, value_name = "VAR=VALUE")]
    pub env: Vec<String>,

    /// Don't inherit environment variables
    #[arg(long)]
    pub no_env: bool,

    /// Environment variable patterns to inherit (comma-separated)
    #[arg(
        long,
        value_name = "PATTERNS",
        default_value = "PATH,HOME,USER,SHELL,TERM,WT_PROFILE_ID,PSMODULEPATH",
        env = "VTERM_ENV_INHERIT"
    )]
    pub env_inherit: String,
}

impl Args {
    /// Parse additional environment variables from -e flags
    pub fn parse_extra_env(&self) -> anyhow::Result<Vec<(String, String)>> {
        let mut result = Vec::new();
        for env_str in &self.env {
            let parts: Vec<&str> = env_str.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid environment variable format: '{}'. Expected VAR=VALUE",
                    env_str
                ));
            }
            result.push((parts[0].to_string(), parts[1].to_string()));
        }
        Ok(result)
    }

    /// Parse environment inheritance patterns
    pub fn env_patterns(&self) -> Vec<String> {
        self.env_inherit
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extra_env() {
        let args = Args {
            select: false,
            add: None,
            profile: None,
            working_dir: None,
            server: "wss://localhost:5153".to_string(),
            env: vec!["FOO=bar".to_string(), "BAZ=qux".to_string()],
            no_env: false,
            env_inherit: "PATH".to_string(),
        };

        let env = args.parse_extra_env().unwrap();
        assert_eq!(env.len(), 2);
        assert_eq!(env[0], ("FOO".to_string(), "bar".to_string()));
        assert_eq!(env[1], ("BAZ".to_string(), "qux".to_string()));
    }

    #[test]
    fn test_env_patterns() {
        let args = Args {
            select: false,
            add: None,
            profile: None,
            working_dir: None,
            server: "wss://localhost:5153".to_string(),
            env: vec![],
            no_env: false,
            env_inherit: "PATH,HOME,USER".to_string(),
        };

        let patterns = args.env_patterns();
        assert_eq!(patterns, vec!["PATH", "HOME", "USER"]);
    }
}
