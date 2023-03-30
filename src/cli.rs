use clap::Parser; // ValueEnum

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Action {
    /// Run installation
    Install {
        /// path to files
        #[clap(short, long)]
        file: std::path::PathBuf,
        /// if specified, only run this stage
        #[clap(short, long)]
        stage: Option<String>,
    },
    /// Check installation
    Check {
        /// path to files
        #[clap(short, long)]
        file: std::path::PathBuf,
        /// if specified, only check this stage
        #[clap(short, long)]
        stage: Option<String>,
    },
}

// struct for clap CLI args
#[derive(Debug, Parser)]
#[clap(version = "0.1")]
pub struct Opts {
    /// remote SSH host
    #[clap(long, default_value = "", env = "REMOTE_SSH_HOST")]
    pub remote_host: String,
    /// remote SSH user
    #[clap(long, default_value = "root", env = "REMOTE_SSH_USER")]
    pub remote_user: String,
    /// remote SSH port
    #[clap(long, default_value = "22", env = "REMOTE_SSH_PORT")]
    pub remote_port: u16,
    /// path to id_rsa file
    #[clap(long, default_value = "~/.ssh/id_rsa", env = "REMOTE_SSH_KEY_FILE")]
    pub remote_key_file: String,

    /// Action
    #[command(subcommand)]
    pub action: Action,
    /// Log Level
    #[clap(env = "RUST_LOG", default_value = "info")]
    rust_log: Option<String>,
}

impl Opts {
    pub fn into_ssh(&self) -> crate::config::Ssh {
        crate::config::Ssh {
            remote_host: if self.remote_host.len() > 0 {
                Some(self.remote_host.clone())
            } else {
                None
            },
            remote_user: if self.remote_user.len() > 0 {
                Some(self.remote_user.clone())
            } else {
                None
            },
            remote_port: if self.remote_port > 0 {
                Some(self.remote_port.clone())
            } else {
                None
            },
            remote_key_file: if self.remote_key_file.len() > 0 {
                Some(self.remote_key_file.clone())
            } else {
                None
            },
        }
    }
}
