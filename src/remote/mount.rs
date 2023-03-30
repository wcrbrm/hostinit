use crate::prelude::*;

#[derive(Debug, Default, Deserialize)]
pub struct MountOptions {
    /// Destination folder to be mounted, i.e. /data
    /// the biggest free device that is not mounted is selected
    pub to: String,
}
impl MountOptions {
    pub fn new(to: &str) -> Self {
        MountOptions { to: to.to_string() }
    }
}

#[derive(Debug, Deserialize)]
pub struct BlockDevice {
    pub name: String,
    #[serde(rename = "maj:min")]
    pub majmin: String,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub rm: bool,
    pub size: String,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub ro: bool,
    #[serde(rename = "type")]
    pub blocktype: String,
    pub mountpoints: Vec<serde_json::Value>,
    pub children: Option<Vec<BlockDevice>>,
}

impl BlockDevice {
    fn size(&self) -> u64 {
        if self.size.ends_with("T") {
            let num = self.size.trim_end_matches('T').parse::<u64>().unwrap_or(0);
            return num * 1024 * 1024 * 1024 * 1024;
        }
        if self.size.contains("G") {
            let num = self.size.trim_end_matches('G').parse::<u64>().unwrap_or(0);
            return num * 1024 * 1024 * 1024;
        }
        if self.size.contains("M") {
            let num = self.size.trim_end_matches('M').parse::<u64>().unwrap_or(0);
            return num * 1024 * 1024;
        }
        if self.size.contains("K") {
            let num = self.size.trim_end_matches('K').parse::<u64>().unwrap_or(0);
            return num * 1024;
        }
        self.size.parse::<u64>().unwrap_or(0)
    }

    fn has_children(&self) -> bool {
        match &self.children {
            Some(children) => children.len() > 0,
            None => false,
        }
    }

    pub fn is_busy(&self) -> bool {
        if self.blocktype != "disk" || self.has_children() {
            return true;
        }
        if self.mountpoints.len() == 0 {
            return false;
        }
        if self.mountpoints.len() == 1 {
            let v = &self.mountpoints[0];
            if v.is_null() {
                return false;
            }
        }
        return true;
    }
}

#[derive(Debug, Deserialize)]
struct LsBlkOutput {
    blockdevices: Vec<BlockDevice>,
}

impl LsBlkOutput {
    pub fn get_mounted_to(&self, to: String) -> Option<&BlockDevice> {
        for device in &self.blockdevices {
            for mountpoint in &device.mountpoints {
                if mountpoint.is_null() {
                    continue;
                }
                let path = mountpoint.as_str().unwrap_or("");
                if path == to {
                    return Some(device);
                }
            }
        }
        None
    }

    pub fn get_biggest_unmounted(&self) -> Option<&BlockDevice> {
        let mut biggest: Option<&BlockDevice> = None;
        for device in &self.blockdevices {
            if device.is_busy() {
                continue;
            }
            match biggest {
                None => {
                    biggest = Some(device);
                    continue;
                }
                Some(x) => {
                    if device.size() > x.size() {
                        biggest = Some(device);
                    }
                }
            }
        }
        biggest
    }
}

#[instrument(skip(client))]
async fn mounting(
    client: &Client,
    name: &str,
    to: &str,
    blocktype: &str,
    fs_type: &str,
) -> anyhow::Result<()> {
    // mkfs -t ext4 /dev/sdc
    let cmd_mkfs = format!("sudo mkfs -t {} /dev/{} 2>&1", fs_type, name);
    run(&client, &cmd_mkfs).await?;

    // mkdir -p /data2
    let cmd_mkdir = format!("sudo mkdir -p {} 2>&1", to);
    run(&client, &cmd_mkdir).await?;

    // mount /dev/sdc /data2
    let cmd_mount = format!("sudo mount /dev/{} {} 2>&1", name, to);
    run(&client, &cmd_mount).await?;
    Ok(())
}

#[instrument(skip(client))]
async fn update_fstab(client: &Client, name: &str, to: &str, fs_type: &str) -> anyhow::Result<()> {
    // cp /etc/fstab /etc/fstab.bak
    let cmd_fstab_b = "sudo cp /etc/fstab /etc/fstab.bak";
    run(&client, cmd_fstab_b).await?;

    // echo "/dev/sdc       /data   ext4    defaults,nofail        0       0" >> /etc/fstab
    let cmd_fstab = format!(
        "sudo sh -c 'echo \"/dev/{}       {}   {}    defaults,nofail        0       0\" >> /etc/fstab'",
        name, to, fs_type
    );
    run(&client, &cmd_fstab).await?;
    Ok(())
}

#[instrument(skip(client))]
pub async fn on_install(client: &Client, opt: &MountOptions) -> anyhow::Result<()> {
    let result = client.execute("lsblk -J").await?;
    let devices: LsBlkOutput = serde_json::from_str::<LsBlkOutput>(&result.output)?;
    for x in &devices.blockdevices {
        if !x.is_busy() {
            debug!("{} {} is not busy\n", x.name, x.size);
        }
    }
    let found = match &devices.get_mounted_to(opt.to.clone()) {
        Some(device) => {
            info!("folder {} is already used by {}\n", opt.to, device.name);
            device.clone()
        }
        None => {
            let found = devices
                .get_biggest_unmounted()
                .context("failed to find target block device")?;
            mounting(client, &found.name, &opt.to, &found.blocktype, "ext4").await?;
            found.clone()
        }
    };

    // check if the device is already mounted in fstab, and update it
    let device_name = format!("/dev/{}", found.name);
    let is_in_fstab = run(&client, "cat /etc/fstab")
        .await?
        .output
        .contains(&device_name);
    if !is_in_fstab {
        update_fstab(client, &found.name, &opt.to, "ext4").await?;
    } else {
        info!("{} is already in fstab", device_name);
    }

    Ok(())
}

// return types: ready for install, installed
#[instrument(skip(client))]
pub async fn on_check(client: &Client, opt: &MountOptions) -> anyhow::Result<Status> {
    let mut success = vec![];
    let mut fail = vec![];
    let result = client.execute("lsblk -J").await?;
    let devices: LsBlkOutput = serde_json::from_str::<LsBlkOutput>(&result.output)?;
    for x in &devices.blockdevices {
        if !x.is_busy() {
            info!("{} {} is not busy\n", x.name, x.size);
        }
    }
    let device_name = match &devices.get_mounted_to(opt.to.clone()) {
        Some(device) => {
            success.push(format!("folder {} is used by {}", opt.to, device.name));
            format!("/dev/{}", device.name)
        }
        None => {
            let Some(found) = devices
                .get_biggest_unmounted()
                else {
                    fail.push("failed to find target block device".to_string());
                    return Ok(Status::NotInstalled{ success, fail })
                };
            format!("/dev/{}", found.name)
        }
    };
    let is_in_fstab = run(&client, "cat /etc/fstab")
        .await?
        .output
        .contains(&device_name);
    if !is_in_fstab {
        fail.push(format!("device {} is not in fstab", device_name));
    } else {
        success.push(format!("device {} is in fstab", device_name));
    }
    Ok(Status::new(success, fail))
}
