use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct KeysOptions {
    pub sync: Vec<String>,
    pub perm: Option<String>,
}

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &KeysOptions) -> anyhow::Result<()> {
    // syncing each local key with the remote location
    for file in &opt.sync {
        let local_path = crate::connect::tilde_with_context(&file, dirs::home_dir);
        let contents = std::fs::read_to_string(&local_path)?;

        let cmd = format!("echo '{}' > {}", contents, file);
        run(&client, &cmd).await?;

        if let Some(perm) = &opt.perm {
            let cmd = format!("chmod {} {}", perm, file);
            run(&client, &cmd).await?;
        }
    }
    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &KeysOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];
    for file in &opt.sync {
        let cmd = format!("ls -1 {}", file);
        match silent(&client, &cmd).await {
            Ok(output) => {
                if output.exit_status == 0 {
                    success.push(format!("{} found", file));
                } else {
                    fail.push(format!("{} missing", file));
                }
            }
            Err(_) => {
                fail.push(format!("{} missing", file));
            }
        }
    }
    Ok(Status::new(success, fail))
}
