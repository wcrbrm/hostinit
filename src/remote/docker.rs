use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct DockerOptions {
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct DockerConfig {
    #[serde(rename = "data-root")]
    data_root: String,
    #[serde(rename = "log-driver")]
    log_driver: String,
}

impl DockerConfig {
    pub fn new(data_root: &str) -> Self {
        DockerConfig {
            data_root: format!("{}/docker", data_root.trim_end_matches('/')),
            log_driver: "json-file".to_string(),
        }
    }
}

const GPG_PATH: &str = "/etc/apt/keyrings/docker.gpg";
const SOURCES_LIST_PATH: &str = "/etc/apt/sources.list.d/docker.list";
const DAEMON_CONFIG_PATH: &str = "/etc/docker/daemon.json";

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &DockerOptions) -> anyhow::Result<()> {
    let os_str = match osinfo(client).await {
        Os::Debian => "debian",
        Os::Ubuntu => "ubuntu",
        _ => bail!("unsupported OS"),
    };

    if let Some(path) = &opt.path {
        if file_exists(client, DAEMON_CONFIG_PATH).await {
            debug!("docker config already exists, skipping");
        } else {
            // if not, overwrite it
            let cmd1 = "sudo mkdir -p /etc/docker";
            run(&client, &cmd1).await?;

            let cmd = format!(
                "sudo bash -c 'echo {:?} > {}'",
                serde_json::to_string(&DockerConfig::new(&path))?,
                DAEMON_CONFIG_PATH,
            );
            run(client, &cmd).await?;
        }
    }

    if !file_exists(client, GPG_PATH).await {
        // 2. install GPG key
        run(client, "sudo mkdir -m 0755 -p /etc/apt/keyrings").await?;

        let origin = format!("https://download.docker.com/linux/{}/gpg", os_str);
        let cmd = format!("curl -fsSL {} | sudo gpg --dearmor -o {}", origin, GPG_PATH);
        run(client, &cmd).await?;
        // chmod a+r /etc/apt/keyrings/docker.gpg ?
    }

    if !file_exists(client, SOURCES_LIST_PATH).await {
        // 3. setup apt repo
        let cmd = format!(
            r#"echo "deb [arch="$(dpkg --print-architecture)" signed-by={}] https://download.docker.com/linux/{} \
               "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | sudo tee {} > /dev/null"#,
            GPG_PATH, os_str, SOURCES_LIST_PATH
        );
        run(client, &cmd).await?;
    }

    run(client, "export DEBIAN_FRONTEND=noninteractive").await?;
    run(client, "sudo apt-get update 2>&1").await?;

    let mut packages = Vec::new();
    match which(client, "docker --version 2>&1").await {
        Ok(_) => (),
        Err(_) => packages.push(
            "docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin",
        ),
    };
    if !packages.is_empty() {
        let cmd = format!("sudo apt-get install -y {} 2>&1", packages.join(" "));
        run(client, &cmd).await?;
    }
    silent(client, "sudo usermod -aG docker $USER 2>&1").await?;
    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &DockerOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];

    if let Os::Unsupported = osinfo(client).await {
        bail!("unsupported OS");
    };

    match which(client, "docker --version 2>&1").await {
        Ok(res) => success.push(res),
        Err(res) => fail.push(res.to_string()),
    };
    // match which(client, "docker-compose --version 2>&1").await {
    //     Ok(res) => success.push(res),
    //     Err(res) => fail.push(res.to_string()),
    // };

    if file_exists(client, "/etc/apt/keyrings/docker.gpg").await {
        success.push("docker gpg key ok".to_string());
    } else {
        fail.push("missing docker gpg key".to_string());
    }
    if file_exists(client, "/etc/apt/sources.list.d/docker.list").await {
        success.push("/etc/apt/sources.list.d/docker.list ok".to_string());
    } else {
        fail.push("missing /etc/apt/sources.list.d/docker.list".to_string());
    }
    if some_output(client, "cat /etc/group | grep docker | grep $USER").await {
        success.push("current user is in docker group".to_string());
    } else {
        fail.push("current user is not in docker group".to_string());
    }

    Ok(Status::new(success, fail))
}
