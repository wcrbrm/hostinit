use crate::system::apt::AptOptions;
use crate::system::mkdir::MkdirOptions;
use crate::system::mount::MountOptions;
use serde::Deserialize;
use std::collections::BTreeMap as Map;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ssh: Option<Ssh>,
    pub stages: Map<String, Stage>,
}

#[derive(Debug, Deserialize)]
pub struct Ssh {
    pub remote_host: Option<String>,
    pub remote_user: Option<String>,
    pub remote_port: Option<u16>,
    pub remote_key_file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Stage {
    pub mount: Option<MountOptions>,
    pub mkdir: Option<MkdirOptions>,
    pub apt: Option<AptOptions>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let contents = r#"
[stages]

[stages.disk]
mount = { to = "/data" }
mkdir = { perm = "0777", sudo = true, folders = ["/data/exchange", "/data/weblogs", "/data/logs", "/data/webcache" ] }

[stages.essentials]
apt = { install = [ "gnupg", "ca-certificates", "build-essentials", "curl", "jq", "vim", "vifm" ] }

[stages.docker]
docker = { path = "/data" }
terraform
    "#;

        let config: Config = toml::from_str(&contents).unwrap();
        println!("{:?}", config);
    }
}
