pub mod cli;
pub mod config;
pub mod logging;
pub mod prelude;
pub mod system;

use anyhow::Context;
use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use std::path::Path;

pub fn tilde_with_context<SI: ?Sized, P, HD>(input: &SI, home_dir: HD) -> String
where
    SI: AsRef<str>,
    P: AsRef<Path>,
    HD: FnOnce() -> Option<P>,
{
    let input_str = input.as_ref();
    if input_str.starts_with("~") {
        let input_after_tilde = &input_str[1..];
        if input_after_tilde.is_empty() || input_after_tilde.starts_with("/") {
            if let Some(hd) = home_dir() {
                let result = format!("{}{}", hd.as_ref().display(), input_after_tilde);
                result.into()
            } else {
                // home dir is not available
                input_str.into()
            }
        } else {
            // we cannot handle `~otheruser/` paths yet
            input_str.into()
        }
    } else {
        // input doesn't start with tilde
        input_str.into()
    }
}

// get ssh client, combine args and config, config has higher priority
pub async fn get_client(args: config::Ssh, cfg: &config::Config) -> anyhow::Result<Client> {
    let raw_path_key = match &cfg.ssh {
        Some(ssh) => match &ssh.remote_key_file {
            Some(file) => file.to_string(),
            None => args
                .remote_key_file
                .context("no private key file provided")?,
        },
        None => args
            .remote_key_file
            .context("no private key file provided")?,
    };
    let path_key = tilde_with_context(&raw_path_key, dirs::home_dir);
    let private_key =
        std::fs::read_to_string(&path_key).context(format!("invalid private key {}", path_key))?;
    let method = AuthMethod::with_key(&private_key, None);
    let host = match &cfg.ssh {
        Some(ssh) => match &ssh.remote_host {
            Some(host) => host.to_string(),
            None => args.remote_host.unwrap_or("".to_string()),
        },
        None => args.remote_host.unwrap_or("".to_string()),
    };
    let port = match &cfg.ssh {
        Some(ssh) => match &ssh.remote_port {
            Some(port) => *port,
            None => args.remote_port.unwrap_or(22),
        },
        None => args.remote_port.unwrap_or(22),
    };
    let username = match &cfg.ssh {
        Some(ssh) => match &ssh.remote_user {
            Some(host) => host.to_string(),
            None => args.remote_user.unwrap_or("".to_string()),
        },
        None => args.remote_user.unwrap_or("".to_string()),
    };
    Ok(
        Client::connect((host, port), &username, method, ServerCheckMethod::NoCheck)
            .await
            .unwrap(),
    )
}

mod remote {
    use crate::prelude::*;
    //  use crate::system::mount::{self, MountOptions};
    use crate::config::Stage;

    #[instrument(skip(client))]
    pub async fn install(client: &Client, name: &str, stage: &Stage) -> anyhow::Result<()> {
        println!("=== stage: {}", name);

        if let Some(opt) = &stage.mount {
            match crate::system::mount::on_install(client, opt).await {
                Ok(_) => println!("+ mount: OK"),
                Err(e) => error!("- mount: FAILURE {}", e),
            }
        }
        if let Some(opt) = &stage.mkdir {
            match crate::system::mkdir::on_install(client, opt).await {
                Ok(_) => println!("+ mkdir: OK"),
                Err(e) => error!("- mkdir: FAILURE {}", e),
            }
        }
        Ok(())
    }

    #[instrument(skip(client))]
    pub async fn check(client: &Client, name: &str, stage: &Stage) -> anyhow::Result<()> {
        println!("=== stage: {}", name);

        if let Some(opt) = &stage.mount {
            match crate::system::mount::on_check(client, opt).await {
                Ok(status) => println!("+ mount: {:?}", status),
                Err(e) => error!("- mount: {:?}", e),
            }
        }
        if let Some(opt) = &stage.mkdir {
            match crate::system::mkdir::on_check(client, opt).await {
                Ok(status) => println!("+ mkdir: {:?}", status),
                Err(e) => error!("- mkdir: {:?}", e),
            }
        }
        Ok(())
    }
}

use clap::Parser;
use tracing::*;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().unwrap();
    color_eyre::install().unwrap();
    logging::start();

    let args = cli::Opts::parse();
    debug!("{:?}", args);
    let ssh = args.into_ssh();

    match args.action {
        cli::Action::Install { file } => {
            // read toml config from file
            let cfg: config::Config =
                toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
            let client = get_client(ssh, &cfg).await.unwrap();
            for (name, stage) in cfg.stages {
                remote::install(&client, &name, &stage).await.unwrap();
            }
        }
        cli::Action::Check { file } => {
            let cfg: config::Config =
                toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
            let client = get_client(ssh, &cfg).await.unwrap();
            for (name, stage) in cfg.stages {
                remote::check(&client, &name, &stage).await.unwrap();
            }
        }
    }

    Ok(())
}
