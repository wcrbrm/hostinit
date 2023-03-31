pub mod apt;
pub use apt::AptOptions;

pub mod docker;
pub use docker::DockerOptions;

pub mod git;
pub use git::GitOptions;

pub mod keys;
pub use keys::KeysOptions;

pub mod mkdir;
pub use mkdir::MkdirOptions;

pub mod mount;
pub use mount::MountOptions;

pub mod terraform;
pub use terraform::TerraformOptions;

pub mod aws;
pub use aws::AwsOptions;

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
    pub keys: Option<KeysOptions>,
    pub git: Option<GitOptions>,
    pub aws: Option<AwsOptions>,
    pub docker: Option<DockerOptions>,
    pub terraform: Option<TerraformOptions>,
}

#[instrument(skip(client))]
pub async fn install(client: &Client, name: &str, stage: &Stage) -> anyhow::Result<()> {
    println!("= {}", name.yellow());

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
    if let Some(opt) = &stage.keys {
        let alias = "keys";
        match keys::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.git {
        let alias = "git";
        match git::on_install(client, opt).await {
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
    if let Some(opt) = &stage.terraform {
        let alias = "terraform";
        match terraform::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.aws {
        let alias = "aws";
        match aws::on_install(client, opt).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    Ok(())
}

#[instrument(skip(client))]
pub async fn check(client: &Client, name: &str, stage: &Stage) -> anyhow::Result<()> {
    println!("= {}", name.yellow());

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
    if let Some(opt) = &stage.keys {
        let alias = "keys";
        match keys::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.git {
        let alias = "git";
        match git::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    if let Some(opt) = &stage.aws {
        let alias = "aws";
        match aws::on_check(client, opt).await {
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
    if let Some(opt) = &stage.terraform {
        let alias = "terraform";
        match terraform::on_check(client, opt).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    Ok(())
}
