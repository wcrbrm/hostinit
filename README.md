# remote-playbook

Remote SSH playbook interpreter

- Uses playbook in the TOML format
- Doesn't require anything to be installed on a target host (except sudo without password)
- Safely checks the installation remotely
- Provides details logs for every installation step (use `RUST_LOG`)

### Capabilities

This tool can be used to on remote Linux systems, tested on Debians/Ubuntu at this point

- Create remote directories
- Mount external hard drives

### Usage

`remote-playbook check --file <FILE.toml>`
`remote-playbook install --file <FILE.toml>`

### Example

Example of a playbook is below

```
[ssh]
remote_key_file = "~/.ssh/id_rsa"
remote_user = "azureuser"
remote_host = "127.0.0.1"

[stages]

[stages.disk]
mount = { to = "/data" }
mkdir = { perm = "0777", sudo = true, folders = ["/data/exchange", "/data/weblogs", "/data/logs", "/data/webcache" ] }

[stages.docker]
apt = { install = [ "gnupg", "ca-certificates", "build-essentials", "curl", "jq", "vim", "vifm" ] }
docker = { path = "/data" }
terraform
```

### Disclaimer

This is a proof on concept of how Rust can take Ansible responsibilities
