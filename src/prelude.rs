pub use anyhow::{bail, Context};
pub use async_ssh2_tokio::client::{Client, CommandExecutedResult};
pub use serde::{Deserialize, Serialize};
pub use serde_aux::prelude::*;
pub use tracing::*;

pub async fn run(client: &Client, cmd: &str) -> anyhow::Result<CommandExecutedResult> {
    let exec_result: CommandExecutedResult = client.execute(&cmd).await?;
    if exec_result.exit_status == 0 {
        debug!("{} {:?}", cmd, exec_result);
        Ok(exec_result)
    } else {
        error!("{} {:?}", cmd, exec_result);
        Err(anyhow::Error::msg(exec_result.output))
    }
}

pub async fn silent(client: &Client, cmd: &str) -> anyhow::Result<CommandExecutedResult> {
    let exec_result: CommandExecutedResult = client.execute(&cmd).await?;
    if exec_result.exit_status == 0 {
        // debug!("{} {:?}", cmd, exec_result);
        Ok(exec_result)
    } else {
        // error!("{} {:?}", cmd, exec_result);
        Err(anyhow::Error::msg(exec_result.output))
    }
}

#[derive(Debug, Serialize)]
pub enum Status {
    Installed {
        success: Vec<String>,
    },
    NotInstalled {
        success: Vec<String>,
        fail: Vec<String>,
    },
}

impl Status {
    pub fn new(success: Vec<String>, failure: Vec<String>) -> Self {
        if failure.is_empty() {
            Status::Installed { success }
        } else {
            Status::NotInstalled {
                success,
                fail: failure,
            }
        }
    }
}
