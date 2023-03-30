use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct MkdirOptions {
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub sudo: bool,
    pub folders: Vec<String>,
    pub perm: Option<String>,
}

impl MkdirOptions {
    pub fn writtable(input: impl Iterator<Item = impl Into<String>>) -> Self {
        let folders = input.map(|x| x.into()).collect();
        MkdirOptions {
            sudo: true,
            folders,
            perm: Some("0777".to_string()),
        }
    }
}
#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &MkdirOptions) -> anyhow::Result<()> {
    let sudo_prefix = if opt.sudo { "sudo " } else { "" };
    let list = opt.folders.join(" ");
    let cmd1 = format!("{} mkdir -p {}", sudo_prefix, list);
    run(&client, &cmd1).await?;

    let default = "0777".to_string();
    let perm = opt.perm.as_ref().unwrap_or(&default);
    let cmd2 = format!("{} chown -R {} {}", sudo_prefix, perm, list);
    run(&client, &cmd2).await?;
    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &MkdirOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];
    for folder in &opt.folders {
        let cmd = format!("ls -d {}", folder);
        match silent(&client, &cmd).await {
            Ok(output) => {
                if output.exit_status == 0 {
                    success.push(format!("{} found", folder));
                } else {
                    fail.push(format!("{} missing", folder));
                }
            }
            Err(_) => {
                fail.push(format!("{} missing", folder));
            }
        }
    }
    Ok(Status::new(success, fail))
}
