use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct TerraformOptions {}

const GPG_PATH: &str = "/usr/share/keyrings/hashicorp-archive-keyring.gpg";
const SOURCES_LIST_PATH: &str = "/etc/apt/sources.list.d/hashicorp.list";

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &TerraformOptions) -> anyhow::Result<()> {
    if !file_exists(client, GPG_PATH).await {
        // install GPG key
        run(client, "sudo mkdir -m 0755 -p /etc/apt/share/keyrings").await?;

        let origin = "https://apt.releases.hashicorp.com/gpg";
        let cmd = format!("wget -O- {} | sudo gpg --dearmor -o {}", origin, GPG_PATH);
        run(client, &cmd).await?;
    }

    // verify GPG key
    let cmd = format!(
        "gpg --no-default-keyring --keyring {} --fingerprint",
        GPG_PATH,
    );
    run(client, &cmd).await?;

    if !file_exists(client, SOURCES_LIST_PATH).await {
        let lsb_release = run(&client, "lsb_release -cs").await?.output.trim().lines().next().unwrap_or("").to_string();
        // setup apt repo
        let cmd = format!(
            "echo \"deb [signed-by={}] https://apt.releases.hashicorp.com {} main\" | sudo tee {} > /dev/null",
            GPG_PATH,
            lsb_release, 
            SOURCES_LIST_PATH,
        );
        run(client, &cmd).await?;
    }
    run(client, "export DEBIAN_FRONTEND=noninteractive").await?;
    run(client, "sudo apt-get update 2>&1").await?;

    let mut packages = Vec::with_capacity(1);
    match which(client, "terraform --version 2>&1").await {
        Ok(_) => (),
        Err(_) => packages.push("terraform"),
    };
    if !packages.is_empty() {
        let cmd = format!("sudo apt-get install -y {} 2>&1", packages.join(" "));
        run(client, &cmd).await?;
    }

    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &TerraformOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];

    match which(client, "terraform --version 2>&1").await {
        Ok(msg) => {
            let first_line = msg.lines().next().unwrap_or("").to_string();
            success.push(first_line);
        }
        Err(res) => fail.push(res.to_string()),
    };
    if file_exists(client, GPG_PATH).await {
        success.push("gpg key ok".to_string());
    } else {
        fail.push("missing gpg key".to_string());
    }
    if file_exists(client, SOURCES_LIST_PATH).await {
        success.push("sources list ok".to_string());
    } else {
        fail.push(format!("missing {}", SOURCES_LIST_PATH));
    }

    Ok(Status::new(success, fail))
}
