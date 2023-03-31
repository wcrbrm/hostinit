use crate::remote::Stage;

use serde::Deserialize;
use std::collections::BTreeMap as Map;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ssh: Option<Ssh>,
    pub stages: Map<String, Stage>,
    pub aliases: Option<Map<String, String>>,
    // pub exports: Option<Map<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct Ssh {
    pub remote_host: Option<String>,
    pub remote_user: Option<String>,
    pub remote_port: Option<u16>,
    pub remote_password: Option<String>,
    pub remote_key_file: Option<String>,
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
apt = { install = [ "gnupg", "ca-certificates", "build-essential", "curl", "jq", "vim", "software-properties-common" ] }

[stages.docker]
docker = { path = "/data" }
terraform = {}
    "#;

        let config: Config = toml::from_str(&contents).unwrap();
        println!("{:?}", config);
    }
}
