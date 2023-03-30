pub mod apt;
pub use apt::AptOptions;

pub mod docker;
pub use docker::DockerOptions;

pub mod mkdir;
pub use mkdir::MkdirOptions;

pub mod mount;
pub use mount::MountOptions;

// use crate::prelude::*;
use async_ssh2_tokio::client::Client;
use color_eyre::owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::*;

#[derive(Debug, Deserialize)]
pub struct Stage {
    pub mount: Option<MountOptions>,
    pub mkdir: Option<MkdirOptions>,
    pub apt: Option<AptOptions>,
    pub docker: Option<DockerOptions>,
}

#[instrument(skip(client))]
pub async fn install(client: &Client, name: &str, stage: &Stage) -> anyhow::Result<()> {
    println!("=== {}", name.yellow());

    if let Some(opt) = &stage.mount {
        let alias = "mount";
        match mount::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.mkdir {
        let alias = "mkdir";
        match mkdir::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.apt {
        let alias = "apt";
        match apt::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.docker {
        let alias = "docker";
        match docker::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    Ok(())
}

#[instrument(skip(client))]
pub async fn check(client: &Client, name: &str, stage: &Stage) -> anyhow::Result<()> {
    println!("=== {}", name.yellow());

    if let Some(opt) = &stage.mount {
        let alias = "mount";
        match mount::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.mkdir {
        let alias = "mkdir";
        match mkdir::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.apt {
        let alias = "apt";
        match apt::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.docker {
        let alias = "docker";
        match docker::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    Ok(())
}
