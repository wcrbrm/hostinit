use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct GitOptions {
    pub to: String,
    pub clone: String,
}

use std::path::Path;

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &GitOptions) -> anyhow::Result<()> {
    // syncing each local key with the remote location
    if !file_exists(client, &opt.to).await {
        let base_name = Path::new(&opt.to)
            .file_name()
            .context("base name")?
            .to_str()
            .context("path to str")?;

        let parent = Path::new(&opt.to)
            .parent()
            .context("parent dir")?
            .to_str()
            .context("path to str")?;

        let mut dest = format!("{}/{}", parent, base_name);
        let home_dir = match dirs::home_dir() {
            Some(path) => path.to_str().context("path to str")?.to_string(),
            None => "".to_string(),
        };
        if parent != home_dir {
            run(&client, &format!("mkdir -p {} 2>&1", parent)).await?;
        } else {
            // we are cloning to home dir, so we need to use relative path
            dest = base_name.to_string();
        }

        let ssh_opts = "ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no";
        let cmd = format!(
            "GIT_SSH_COMMAND=\"{}\" GIT_TERMINAL_PROMPT=0 git clone {} {} 2>&1",
            ssh_opts, opt.clone, dest,
        );
        run(&client, &cmd).await?;
    }

    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &GitOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];

    if file_exists(client, &opt.to).await {
        success.push(format!("{} exists", opt.to));
        let git_config = format!("{}/.git/config", opt.to);
        if file_exists(client, &git_config).await {
            success.push(format!("{} exists", opt.to));
        } else {
            fail.push(format!("{} missing", opt.to));
        }
    } else {
        fail.push(format!("{} missing", opt.to));
    }

    Ok(Status::new(success, fail))
}
