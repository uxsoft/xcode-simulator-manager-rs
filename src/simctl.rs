use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

pub type Udid = String;

#[derive(Debug, Clone)]
pub struct Simulator {
    pub udid: Udid,
    pub name: String,
    pub runtime: String,
    pub state: DeviceState,
    pub is_available: bool,
    pub data_path: String,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Booted,
    Shutdown,
    Other,
}

impl DeviceState {
    fn parse(s: &str) -> Self {
        match s {
            "Booted" => Self::Booted,
            "Shutdown" => Self::Shutdown,
            _ => Self::Other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Booted => "Booted",
            Self::Shutdown => "Shutdown",
            Self::Other => "Other",
        }
    }
}

#[derive(Deserialize)]
struct ListResponse {
    devices: BTreeMap<String, Vec<RawDevice>>,
}

#[derive(Deserialize)]
struct RawDevice {
    udid: String,
    name: String,
    state: String,
    #[serde(rename = "isAvailable")]
    is_available: bool,
    #[serde(rename = "dataPath")]
    data_path: String,
}

pub fn list_devices() -> Result<Vec<Simulator>> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devices", "--json"])
        .output()
        .context("failed to invoke `xcrun simctl list devices --json`")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("simctl list failed: {}", stderr.trim()));
    }
    let parsed: ListResponse =
        serde_json::from_slice(&output.stdout).context("failed to parse simctl JSON")?;

    let mut sims = Vec::new();
    for (runtime_id, devices) in parsed.devices {
        let runtime = pretty_runtime(&runtime_id);
        for d in devices {
            sims.push(Simulator {
                udid: d.udid,
                name: d.name,
                runtime: runtime.clone(),
                state: DeviceState::parse(&d.state),
                is_available: d.is_available,
                data_path: d.data_path,
                size_bytes: None,
            });
        }
    }
    Ok(sims)
}

fn pretty_runtime(id: &str) -> String {
    // e.g. "com.apple.CoreSimulator.SimRuntime.iOS-26-0" -> "iOS 26.0"
    const PREFIX: &str = "com.apple.CoreSimulator.SimRuntime.";
    let trimmed = id.strip_prefix(PREFIX).unwrap_or(id);
    let mut parts = trimmed.splitn(2, '-');
    let os = parts.next().unwrap_or(trimmed);
    let ver = parts.next().unwrap_or("").replace('-', ".");
    if ver.is_empty() {
        os.to_string()
    } else {
        format!("{os} {ver}")
    }
}

pub fn shutdown(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "shutdown", udid])
        .output()
        .context("failed to invoke `xcrun simctl shutdown`")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // simctl returns non-zero if already shut down — treat that as success.
        if stderr.contains("Unable to shutdown device in current state: Shutdown") {
            return Ok(());
        }
        return Err(anyhow!("simctl shutdown {udid} failed: {}", stderr.trim()));
    }
    Ok(())
}

pub fn delete(udids: &[Udid]) -> Result<()> {
    if udids.is_empty() {
        return Ok(());
    }
    let mut cmd = Command::new("xcrun");
    cmd.args(["simctl", "delete"]);
    for u in udids {
        cmd.arg(u);
    }
    let output = cmd
        .output()
        .context("failed to invoke `xcrun simctl delete`")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("simctl delete failed: {}", stderr.trim()));
    }
    Ok(())
}
