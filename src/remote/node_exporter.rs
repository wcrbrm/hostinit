use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct NodeExporterOptions {}

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &NodeExporterOptions) -> anyhow::Result<()> {
    let cmd = "docker run --name node-exporter --restart=always -d --net=\"host\" --pid=\"host\" -v \"/:/host:ro,rslave\" quay.io/prometheus/node-exporter:latest --path.rootfs=/host";
    client.execute(cmd).await?;
    Ok(())
}

// return types: ready for install, installed
#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &NodeExporterOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];

    let result = client
        .execute("docker ps --filter=name=node-exporter --format '{{.ID}}'")
        .await?;
    if result.output.is_empty() {
        fail.push("node-exporter is not running".to_string());
    } else {
        success.push(format!("node-exporter is {}", result.output.trim()));
    }
    Ok(Status::new(success, fail))
}
