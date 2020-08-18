use eyre::{eyre, Context, Error, Result};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::path::PathBuf;

use super::imds;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Limits {
    pub virtual_machine: VirtualMachine,
    pub disks: HashMap<String, Disk>,
}

pub fn get_limits(
    os_disk: Disk,
    data_disks: &Vec<imds::DataDisk>,
    disk_skus: Vec<Disk>,
) -> Result<HashMap<String, Disk>> {
    let mut limits: HashMap<String, Disk> = HashMap::new();
    limits.insert("/dev/sda".to_string(), os_disk);

    for disk in data_disks {
        let size = disk.disk_size_gb.parse::<u64>()?;
        let storage_account_type = &disk.managed_disk.storage_account_type;
        let disk_sku = get_disk_sku(&disk_skus, &size, &storage_account_type)?;

        // TODO(ace): clean this up...maybe shell to readlink -f?
        // normalization without following the symlink seems
        // strangely difficult.
        let device_file = fs::read_link(format!("/dev/disk/azure/scsi1/lun{}", &disk.lun))
            .wrap_err_with(|| "failed to read link")?;

        let device_file: PathBuf =
            RelativePath::new(&format!("/dev/disk/azure/scsi1/{}", device_file.display(),))
                .normalize()
                .to_path("/");

        let device_file = match device_file.to_owned().into_os_string().into_string() {
            Err(e) => {
                return Err(eyre!(
                    "failed to convert path to friendly udev label: {:?}; err: {:?}",
                    &device_file,
                    &e,
                ))
            }
            Ok(s) => s,
        };

        limits.insert(device_file, disk_sku);
    }

    Ok(limits)
}

pub async fn get_vm_sku(
    token: &str,
    subscription_id: &str,
    location: &str,
    name: &str,
) -> Result<VirtualMachine> {
    let mut filtered = list_skus(token, subscription_id)
        .await?
        .value
        .into_iter()
        .filter(|sku| sku.resource_type == "virtualMachines")
        .filter(|sku| sku.locations.len() > 0 && sku.locations[0] == location)
        .filter(|sku| sku.name == name)
        .map(|sku| VirtualMachine::try_from(sku))
        .collect::<Result<Vec<VirtualMachine>>>()?;

    match filtered.len() {
        1 => Ok(filtered.pop().unwrap()),
        n => {
            return Err(eyre!(
                "expected single matching vm sku but found {}. matches: {:#?}",
                &n,
                &filtered,
            ))
        }
    }
}

pub async fn list_disk_skus(
    token: &str,
    subscription_id: &str,
    location: &str,
) -> Result<Vec<Disk>> {
    list_skus(token, subscription_id)
        .await?
        .value
        .into_iter()
        .filter(|sku| sku.resource_type == "disks")
        .filter(|sku| sku.locations.len() > 0 && sku.locations[0] == location)
        .filter(|res| res.tier != Some("Ultra".to_string())) // Need to support ultra, it has different range-based structure
        .map(|sku| Disk::try_from(sku))
        .collect::<Result<Vec<Disk>>>()
}

pub fn get_disk_sku(skus: &Vec<Disk>, size: &u64, storage_account_type: &str) -> Result<Disk> {
    let mut filtered = skus
        .iter()
        .filter(|sku| &sku.storage_account_type == storage_account_type)
        .filter(|sku| size > &sku.min_size_gb && size <= &sku.max_size_gb)
        .cloned()
        .collect::<Vec<Disk>>();

    if filtered.len() < 1 {
        return Err(eyre!(
            "no matching sku found for disk with size: {}, storage type: {}",
            &size,
            &storage_account_type,
        ));
    }

    if filtered.len() > 1 {
        return Err(eyre!(
            "multiple matching sku found for disk with size: {}, storage type: {}, matches: {:#?}",
            &size,
            &storage_account_type,
            &filtered,
        ));
    }

    Ok(filtered.pop().unwrap())
}

pub async fn list_skus(token: &str, subscription_id: &str) -> Result<ResourceList> {
    let res = reqwest::Client::new()
        .get(&format!(
            "https://management.azure.com/subscriptions/{}/providers/Microsoft.Compute/skus",
            subscription_id
        ))
        .header("Authorization", &format!("Bearer {}", token))
        .query(&[("api-version", "2019-04-01")])
        .send()
        .await
        .wrap_err_with(|| "failed to fetch resource skus")?
        .text()
        .await
        .wrap_err_with(|| "failed to receive resource skus response")?;

    let res: ResourceList = serde_json::from_str(&res[..])
        .wrap_err_with(|| format!("failed to parse resource skus from response: {}", &res))?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceList {
    pub value: Vec<Resource>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    pub family: Option<String>,
    pub location_info: Vec<LocationInfo>,
    pub locations: Vec<String>,
    pub name: String,
    pub resource_type: String,
    pub size: Option<String>,
    pub tier: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub name: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    pub location: String,
    pub zone_details: Vec<ZoneDetail>,
    pub zones: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneDetail {
    #[serde(rename = "Name")]
    pub name: Vec<String>,
    pub capabilities: Vec<Capability>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualMachine {
    pub name: String,
    pub location: String,
    pub max_iops: u64,
    pub max_bandwidth: u64,
}

impl TryFrom<Resource> for VirtualMachine {
    type Error = Error;

    fn try_from(value: Resource) -> Result<Self, Self::Error> {
        if value.locations.len() < 1 {
            return Err(eyre!("no locations available for provided sku"));
        }

        let name = value.name.clone();
        let location = value.locations[0].clone();
        let mut capabilities: HashMap<String, String> = HashMap::new();
        for cap in value.capabilities.clone() {
            capabilities.entry(cap.name).or_insert(cap.value);
        }

        let max_iops = capabilities
            .get("UncachedDiskIOPS")
            .ok_or(eyre!("failed to find minimum vm sku size: {:#?}", &value))?
            .parse::<u64>()?;

        // TODO(ace): for some reason SKUs API doesn't return bandwidth
        // numbers for b12ms. Everything above b8ms has the same limit,
        // so we can work around it. Probably ask CRP why this is
        // unpopulated. Because we accept the sku name filter before
        // invoking try_from, this only affects users on bad skus like b12ms.
        let max_bandwidth = capabilities
            .get("UncachedDiskBytesPerSecond")
            .ok_or(eyre!("failed to find maximum vm sku bandwidth"))?
            .parse::<u64>()?;

        let sku = VirtualMachine {
            name,
            location,
            max_iops,
            max_bandwidth,
        };

        Ok(sku)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Disk {
    pub location: String,
    pub storage_account_type: String,
    pub min_size_gb: u64,
    pub max_size_gb: u64,
    pub max_iops: u64,
    pub max_bandwidth: u64,
}

impl TryFrom<Resource> for Disk {
    type Error = Error;

    fn try_from(value: Resource) -> Result<Self, Self::Error> {
        if value.locations.len() < 1 {
            return Err(eyre!("no locations available for provided sku"));
        }

        let location = value.locations[0].clone();
        let storage_account_type = value.name.clone();
        let mut capabilities: HashMap<String, String> = HashMap::new();
        for cap in value.capabilities.clone() {
            capabilities.entry(cap.name).or_insert(cap.value);
        }

        // Azure provides VM bandwith limits in B/s but disk limits in MB/s
        let base: u64 = 2;
        let exp = |n| -> u64 { n * base.pow(20) };

        let min_size_gb = capabilities
            .get("MinSizeGiB")
            .ok_or(eyre!("failed to find minimum disk sku size"))?
            .parse::<u64>()?;

        let max_size_gb = capabilities
            .get("MaxSizeGiB")
            .ok_or(eyre!("failed to find maximum disk sku size"))?
            .parse::<u64>()?;

        let max_iops = capabilities
            .get("MaxIOps")
            .ok_or(eyre!("failed to find maximum disk sku iops"))?
            .parse::<u64>()?;

        let max_bandwidth = capabilities
            .get("MaxBandwidthMBps")
            .ok_or(eyre!("failed to find maximum disk sku bandwidth"))?
            .parse::<u64>()
            .map(exp)?;

        let sku = Disk {
            location,
            storage_account_type,
            min_size_gb,
            max_size_gb,
            max_iops,
            max_bandwidth,
        };

        Ok(sku)
    }
}
