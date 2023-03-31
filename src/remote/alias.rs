use crate::prelude::*;
use base64::{engine::general_purpose, Engine as _};

#[instrument(skip(client))]
pub async fn on_install(client: &Client, key: &str, value: &str) -> anyhow::Result<()> {
    let cmd_check = format!("cat ~/.bashrc | grep {}= 2>&1", key);
    if let Err(_) = which(client, &cmd_check).await {
        let orig = format!("alias {}='{}'\n", key, value);
        let encoded = general_purpose::STANDARD_NO_PAD.encode(orig.as_bytes());
        let cmd_set = format!("echo {} | base64 -d - >> ~/.bashrc", encoded);
        run(client, &cmd_set).await?;
    };
    Ok(())
}

#[instrument(skip(client))]
pub async fn install(client: &Client, items: &Map<String, String>) -> anyhow::Result<()> {
    println!("= {}", "ALIASES".yellow());
    for (alias, value) in items {
        match on_install(client, alias, value).await {
            Ok(_) => println!("+ {}: {}", alias.green(), "OK".green()),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }
    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, key: &str) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];

    let cmd_check = format!("cat ~/.bashrc | grep {}= 2>&1", key);
    match which(client, &cmd_check).await {
        Ok(res) => success.push(res),
        Err(res) => fail.push(res.to_string().replace("alias: ", "")),
    };
    Ok(Status::new(success, fail))
}

#[instrument(skip(client, items))]
pub async fn check(client: &Client, items: &Map<String, String>) -> anyhow::Result<()> {
    println!("= {}", "ALIASES".yellow());

    for (alias, _) in items {
        match on_check(client, alias).await {
            Ok(status) => status.print(alias),
            Err(e) => println!("- {}: {} {}", alias.red(), "FAILURE".red(), e),
        }
    }

    Ok(())
}
