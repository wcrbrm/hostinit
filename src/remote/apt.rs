use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct AptOptions {
    pub install: Vec<String>,
}

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &AptOptions) -> anyhow::Result<()> {
    let packages = opt.install.join(" ");
    run(client, "export DEBIAN_FRONTEND=noninteractive").await?;
    run(client, "sudo apt-get update 2>&1").await?;
    if let Err(e) = run(
        client,
        &format!("sudo apt-get install -yq {} 2>&1", packages),
    )
    .await
    {
        // parse each "Unable to locate package" row to return better error
        let mut missing = vec![];
        for line in e.to_string().lines() {
            if line.contains("Unable to locate package") {
                let package = line
                    .replace("Unable to locate package ", "")
                    .replace("E: ", "");
                missing.push(package);
            }
        }
        if missing.len() > 0 {
            bail!("Unable to locate: {}", missing.join(", "));
        }
        return Err(e);
    }

    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &AptOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];
    for package in &opt.install {
        let cmd = format!("sudo dpkg -s {} 2>&1", package);
        match silent(client, &cmd).await {
            Ok(output) => {
                if output.exit_status == 0 {
                    if output.output.contains("Status: install ok installed") {
                        success.push(format!("{} ok", package));
                    } else {
                        fail.push(format!("{} missing", package));
                    }
                } else {
                    let errmsg = output.output.replace("dpkg-query: ", "").clone();
                    let first_line = errmsg.lines().next().unwrap_or("").to_string();
                    fail.push(first_line);
                }
            }
            Err(_) => {
                fail.push(format!("{} missing", package));
            }
        }
    }
    Ok(Status::new(success, fail))
}
