pub mod cli;
pub mod config;
pub mod connect;
pub mod logging;
pub mod prelude;
pub mod remote;

use clap::Parser;
use tracing::*;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let _ = dotenv::dotenv();
    color_eyre::install().unwrap();
    logging::start();

    let args = cli::Opts::parse();
    debug!("{:?}", args);
    let ssh = args.into_ssh();

    match args.action {
        cli::Action::Install { file, stage } => {
            // read toml config from file
            let cfg: config::Config =
                toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
            let client = connect::get_client(ssh, &cfg).await.unwrap();
            match stage {
                Some(stage) => {
                    if stage == "aliases" {
                        if let Some(aliases) = &cfg.aliases {
                            remote::alias::install(&client, aliases).await.unwrap();
                        } else {
                            panic!("no aliases declared");
                        }
                    } else {
                        remote::install(&client, &stage, &cfg.stages[&stage])
                            .await
                            .unwrap();
                    }
                }
                None => {
                    for (name, stage) in cfg.stages {
                        remote::install(&client, &name, &stage).await.unwrap();
                    }
                    if let Some(aliases) = &cfg.aliases {
                        remote::alias::install(&client, aliases).await.unwrap();
                    }
                }
            }
        }
        cli::Action::Check { file, stage } => {
            let cfg: config::Config =
                toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
            let client = connect::get_client(ssh, &cfg).await.unwrap();
            match stage {
                Some(stage) => {
                    if stage == "aliases" {
                        if let Some(aliases) = &cfg.aliases {
                            remote::alias::check(&client, aliases).await.unwrap();
                        } else {
                            panic!("no aliases declared");
                        }
                    } else {
                        remote::check(&client, &stage, &cfg.stages[&stage])
                            .await
                            .unwrap();
                    }
                }
                None => {
                    for (name, stage) in cfg.stages {
                        remote::check(&client, &name, &stage).await.unwrap();
                    }
                    if let Some(aliases) = &cfg.aliases {
                        remote::alias::check(&client, aliases).await.unwrap();
                    }
                }
            }
        }
    }

    Ok(())
}
