use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

pub fn new() -> Result<HashMap<String, Disk>> {
    // invoke iostat for 1, 1-second interval with detailed table output
    // and no extra summary
    let output = Command::new("/sysstat/iostat")
        .arg("-xty")
        .arg("-o")
        .arg("JSON")
        .arg("1")
        .arg("1")
        .output()?;

    if !output.status.success() {
        return Err(eyre!(
            "failed to execute iostat. stdout: {:?}, stderr: {:?}.",
            output.stdout,
            output.stderr,
        ));
    }

    let mut map: HashMap<String, Disk> = HashMap::new();
    let mut root: Root = serde_json::from_slice(&output.stdout)?;

    let mut stats: Vec<Statistic> = root
        .sysstat
        .hosts
        .pop()
        .ok_or(eyre!("no iostat hosts found"))?
        .statistics
        .clone();
    let stats = stats.pop().ok_or(eyre!("no iostat stats found"))?;

    for disk in stats.disk {
        map.insert(format!("/dev/{}", &disk.disk_device), disk);
    }

    Ok(map)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub sysstat: Sysstat,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sysstat {
    pub hosts: Vec<Host>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Host {
    pub nodename: String,
    pub sysname: String,
    pub release: String,
    pub machine: String,
    #[serde(rename = "number-of-cpus")]
    pub number_of_cpus: i64,
    pub date: String,
    pub statistics: Vec<Statistic>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistic {
    pub timestamp: String,
    #[serde(rename = "avg-cpu")]
    pub avg_cpu: AverageCpU,
    pub disk: Vec<Disk>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AverageCpU {
    pub user: f64,
    pub nice: f64,
    pub system: f64,
    pub iowait: f64,
    pub steal: f64,
    pub idle: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Disk {
    pub disk_device: String,
    #[serde(rename = "r/s")]
    pub reads_per_second: f64,
    #[serde(rename = "w/s")]
    pub writes_per_second: f64,
    #[serde(rename = "d/s")]
    pub discards_per_second: f64,
    #[serde(rename = "f/s")]
    pub flushes_per_second: f64,
    #[serde(rename = "rkB/s")]
    pub read_kilo_bytes_per_second: f64,
    #[serde(rename = "wkB/s")]
    pub write_kilo_bytes_per_second: f64,
    #[serde(rename = "dkB/s")]
    pub discard_kilo_bytes_per_second: f64,
    #[serde(rename = "rrqm/s")]
    pub read_requests_merged_per_second: f64,
    #[serde(rename = "wrqm/s")]
    pub write_requests_merged_per_second: f64,
    #[serde(rename = "drqm/s")]
    pub discard_requests_merged_per_second: f64,
    #[serde(rename = "rrqm")]
    pub percent_read_requests_merged: f64,
    #[serde(rename = "wrqm")]
    pub percent_write_requests_merged: f64,
    #[serde(rename = "drqm")]
    pub percent_discard_requests_merged: f64,
    #[serde(rename = "r_await")]
    pub read_await: f64,
    #[serde(rename = "w_await")]
    pub write_await: f64,
    #[serde(rename = "d_await")]
    pub discard_await: f64,
    #[serde(rename = "f_await")]
    pub flush_await: f64,
    #[serde(rename = "rareq-sz")]
    pub read_average_request_size: f64,
    #[serde(rename = "wareq-sz")]
    pub write_average_request_size: f64,
    #[serde(rename = "dareq-sz")]
    pub discard_average_request_size: f64,
    #[serde(rename = "aqu-sz")]
    pub average_request_size: f64,
    pub util: f64,
}

impl Disk {
    pub fn iops(&self) -> f64 {
        self.reads_per_second + self.writes_per_second
    }

    pub fn throughput(&self) -> f64 {
        self.read_kilo_bytes_per_second + self.write_kilo_bytes_per_second
    }
}
