use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct DockerStatsOptions {}

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &DockerStatsOptions) -> anyhow::Result<()> {
    let cmd = "docker run -d --name=docker-stats --restart=always -p 9487:9487 -v /var/run/docker.sock:/var/run/docker.sock wywywywy/docker_stats_exporter:latest";
    client.execute(cmd).await?;
    Ok(())
}

// return types: ready for install, installed
#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &DockerStatsOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];

    let result = client
        .execute("docker ps --filter=name=docker-stats --format '{{.ID}}'")
        .await?;
    if result.output.is_empty() {
        fail.push("docker-stats is not running".to_string());
    } else {
        success.push(format!("docker-stats is {}", result.output.trim()));
    }
    Ok(Status::new(success, fail))
}
