use eyre::Result;
use serde::{Deserialize, Serialize};

pub async fn new() -> Result<Metadata> {
    let res = reqwest::Client::new()
        .get("http://169.254.169.254/metadata/instance")
        .header("Metadata", "true")
        .query(&[("api-version", "2019-08-15"), ("format", "json")])
        .send()
        .await?
        .json::<Metadata>()
        .await?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub compute: Compute,
    pub network: Network,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Compute {
    pub az_environment: String,
    pub custom_data: String,
    pub location: String,
    pub name: String,
    pub offer: String,
    pub os_type: String,
    pub placement_group_id: String,
    pub plan: Plan,
    pub platform_fault_domain: String,
    pub platform_update_domain: String,
    pub provider: String,
    pub public_keys: Vec<PublicKey>,
    pub publisher: String,
    pub resource_group_name: String,
    pub resource_id: String,
    pub sku: String,
    pub storage_profile: StorageProfile,
    pub subscription_id: String,
    pub tags: String,
    pub tags_list: Vec<TagsList>,
    pub version: String,
    pub vm_id: String,
    pub vm_scale_set_name: String,
    pub vm_size: String,
    pub zone: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub name: String,
    pub product: String,
    pub publisher: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub key_data: String,
    pub path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageProfile {
    pub data_disks: Vec<DataDisk>,
    pub image_reference: ImageReference,
    pub os_disk: OsDisk,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageReference {
    pub id: String,
    pub offer: String,
    pub publisher: String,
    pub sku: String,
    pub version: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDisk {
    pub caching: String,
    pub create_option: String,
    #[serde(rename = "diskSizeGB")]
    pub disk_size_gb: String,
    pub image: Image,
    pub lun: String,
    pub managed_disk: ManagedDisk,
    pub name: String,
    pub vhd: Vhd,
    pub write_accelerator_enabled: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsDisk {
    pub caching: String,
    pub create_option: String,
    pub diff_disk_settings: DiffDiskSettings,
    #[serde(rename = "diskSizeGB")]
    pub disk_size_gb: String,
    pub encryption_settings: EncryptionSettings,
    pub image: Image,
    pub managed_disk: ManagedDisk,
    pub name: String,
    pub os_type: String,
    pub vhd: Vhd,
    pub write_accelerator_enabled: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffDiskSettings {
    pub option: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionSettings {
    pub enabled: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedDisk {
    pub id: String,
    pub storage_account_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vhd {
    pub uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagsList {
    pub name: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub interface: Vec<Interface>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interface {
    pub ipv4: Ipv4,
    pub ipv6: Ipv6,
    pub mac_address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv4 {
    pub ip_address: Vec<IpAddress>,
    pub subnet: Vec<Subnet>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpAddress {
    pub private_ip_address: String,
    pub public_ip_address: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subnet {
    pub address: String,
    pub prefix: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6 {
    pub ip_address: Vec<::serde_json::Value>,
}
