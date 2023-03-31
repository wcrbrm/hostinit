use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct AwsOptions {
    /// profile to be uploaded
    pub profile: Option<String>,
    /// if profile is uploaded, it can be renamed into a different name
    /// typically used to rename the default profile
    pub rename: Option<String>,
}

fn read_aws_profile_region(profile: &str) -> anyhow::Result<String> {
    let local_path = crate::connect::tilde_with_context("~/.aws/config", dirs::home_dir);
    let aws_config = std::fs::read_to_string(&local_path)?;

    let mut region = String::new();
    let mut found = false;
    let mut reading_profile = false;
    for line in aws_config.lines() {
        if line.trim().len() == 0 || line.trim().starts_with('#') {
            // skip empty lines
            continue;
        }
        if line.starts_with("[") && line.ends_with("]") {
            if reading_profile {
                reading_profile = false;
            }
            if line.trim() == format!("[profile {}]", profile) {
                found = true;
                reading_profile = true;
            }
        } else {
            if reading_profile {
                let (key, value) = match line.find('=') {
                    Some(index) => line.split_at(index),
                    None => continue,
                };
                if key.trim() == "region" {
                    region = value[1..].trim().to_string();
                }
            }
        }
    }

    if !found {
        return Err(anyhow::anyhow!(
            "profile {} not found in ~/.aws/config",
            profile
        ));
    } else if region.is_empty() {
        return Err(anyhow::anyhow!("region not specified for {}", profile));
    }
    Ok(region)
}

fn read_aws_profile_keys(profile: &str) -> anyhow::Result<(String, String)> {
    let local_path = crate::connect::tilde_with_context("~/.aws/credentials", dirs::home_dir);
    let aws_config = std::fs::read_to_string(&local_path)?;
    let mut access_key = String::new();
    let mut secret_key = String::new();
    let mut found = false;
    let mut reading_profile = false;
    for line in aws_config.lines() {
        if line.trim().len() == 0 || line.trim().starts_with('#') {
            // skip empty lines
            continue;
        }
        if line.starts_with("[") && line.ends_with("]") {
            if reading_profile {
                reading_profile = false;
            }
            if line.trim() == format!("[{}]", profile) {
                found = true;
                reading_profile = true;
            }
        } else {
            if reading_profile {
                let (key, value) = match line.find('=') {
                    Some(index) => line.split_at(index),
                    None => continue,
                };
                if key.trim() == "aws_access_key_id" {
                    access_key = value[1..].trim().to_string();
                } else if key.trim() == "aws_secret_access_key" {
                    secret_key = value[1..].trim().to_string();
                }
            }
        }
    }
    if !found {
        return Err(anyhow::anyhow!("profile {} not found", profile));
    } else if access_key.is_empty() {
        return Err(anyhow::anyhow!("access key not specified for {}", profile));
    } else if secret_key.is_empty() {
        return Err(anyhow::anyhow!("secret key not specified for {}", profile));
    }
    Ok((access_key, secret_key))
}

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &AwsOptions) -> anyhow::Result<()> {
    // install aws2 CLI
    if let Err(_) = which(client, "aws --version 2>&1").await {
        let cmd =
            "curl https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip -o awscliv2.zip 2>&1";
        run(&client, cmd).await?;

        let cmd = "unzip -qo awscliv2.zip 2>&1";
        run(&client, cmd).await?;

        let cmd = "sudo ./aws/install 2>&1";
        run(&client, cmd).await?;

        let cmd = "rm -rf awscliv2.zip ./aws 2>&1";
        run(&client, cmd).await?;
    }
    // set up aws profile
    if let Some(p) = &opt.profile {
        let profile = opt.rename.as_ref().unwrap_or(p);

        let (access_key, secret_key) = read_aws_profile_keys(&p)?;
        let region = read_aws_profile_region(&p)?;
        let cmd = format!(
            "aws configure set aws_access_key_id {} --profile {} 2>&1",
            access_key, profile
        );
        run(&client, &cmd).await?;
        let cmd = format!(
            "aws configure set aws_secret_access_key {} --profile {} 2>&1",
            secret_key, profile
        );
        run(&client, &cmd).await?;
        let cmd = format!(
            "aws configure set region {} --profile {} 2>&1",
            region, profile
        );
        run(&client, &cmd).await?;
    }

    Ok(())
}

#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &AwsOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];
    match which(client, "aws --version 2>&1").await {
        Ok(res) => success.push(res),
        Err(res) => fail.push(res.to_string()),
    };

    if let Some(p) = &opt.profile {
        let profile = opt.rename.as_ref().unwrap_or(p);

        let cmd = format!("aws configure --profile {} list", profile);
        match silent(&client, &cmd).await {
            Ok(output) => {
                if output.exit_status == 0 {
                    success.push(format!("profile {} ok", profile));
                } else {
                    fail.push(format!("profile {} missing", profile));
                }
            }
            Err(_) => {
                fail.push(format!("profile {} missing", profile));
            }
        }
    }
    Ok(Status::new(success, fail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn it_reads_keys() {
        let (key, secret) = read_aws_profile_keys("dev2@atlant").unwrap();
        assert!(key.len() > 10);
        assert!(secret.len() > 20);

        let region = read_aws_profile_region("dev2@atlant").unwrap();
        assert_eq!(region, "eu-central-1");
    }
}
