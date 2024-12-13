use std::time::Duration;

use anyhow::{anyhow, bail};

use crate::config::Config;

pub fn reader(config: &Config) -> tokio::process::Command {
    let mut journalctl_command = tokio::process::Command::new("journalctl");
    journalctl_command.args(["-f", "-u", &config.unit, "--no-pager"]);
    if let Some(since) = config.since.as_ref() {
        journalctl_command.arg("--since");
        journalctl_command.arg(since);
    }
    journalctl_command
}

pub async fn get_unit_start_date(config: &Config) -> anyhow::Result<String> {
    let systemctl_process = tokio::process::Command::new("systemctl")
        .args(["show", "-p", "ExecMainStartTimestamp", &config.unit])
        .output();

    let timeout = tokio::time::sleep(Duration::from_secs(3));

    tokio::select! {
        data = systemctl_process => {
            let data = data.map(|v| String::from_utf8_lossy(&v.stdout).to_string())?;

            let (_, timestamp) = data
                .split_once("=")
                .ok_or(anyhow!("Unexpected systemctl output"))?;

            let timestamp = timestamp.trim_end_matches('\n').to_string();

            Ok(timestamp)
        }
        _ = timeout => {
            bail!("Timed out trying to read unit start date");
        }
    }
}
