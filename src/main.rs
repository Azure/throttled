use eyre::{Context, Result};
use hyper::Server;

mod imds;
mod iostat;
mod kube;
mod metrics;
mod oauth;
mod resource;
mod server;

use oauth::OAuthResponse;
use resource::{get_disk_sku, get_limits, get_vm_sku, list_disk_skus};
use server::MetricService;

fn main() -> Result<()> {
    smol::run(async {
        let azure_json = kube::new()?;
        let meta = imds::new().await?;

        let token: OAuthResponse;

        match azure_json.aad_client_id.as_str() {
            "msi" => {
                token = {
                    oauth::get_msi_token(&azure_json.user_assigned_identity_id)
                        .await
                        .wrap_err_with(|| "failed to get msi token")?
                }
            }
            _ => {
                token = {
                    oauth::get_sp_token(
                        &azure_json.aad_client_id,
                        &azure_json.aad_client_secret,
                        &azure_json.tenant_id,
                        "https://management.azure.com",
                    )
                    .await
                    .wrap_err_with(|| "failed to get sp token")?
                }
            }
        }

        let vm_size = meta.compute.vm_size;
        let location = azure_json.location;

        let os_disk_size = meta
            .compute
            .storage_profile
            .os_disk
            .disk_size_gb
            .parse::<u64>()?;

        let os_disk_storage_type = meta
            .compute
            .storage_profile
            .os_disk
            .managed_disk
            .storage_account_type;

        let vm_limit = get_vm_sku(
            &token.access_token,
            &azure_json.subscription_id,
            &location,
            &vm_size,
        )
        .await?;

        let disk_skus =
            list_disk_skus(&token.access_token, &azure_json.subscription_id, &location).await?;
        let os_disk_sku = get_disk_sku(&disk_skus, &os_disk_size, &os_disk_storage_type)?;
        let data_disks = meta.compute.storage_profile.data_disks;
        let disk_limits = get_limits(os_disk_sku, &data_disks, disk_skus)?;

        let log_limits = resource::Limits {
            virtual_machine: vm_limit.clone(),
            disks: disk_limits.clone(),
        };

        println!("{}", serde_json::to_string_pretty(&log_limits)?);

        let metrics = metrics::Tracker::new()?;

        let metrics_server = MetricService {
            metrics,
            vm_limit,
            disk_limits,
        };

        let addr = ([0, 0, 0, 0], 8080).into();

        let server = Server::bind(&addr).serve(metrics_server);

        server.await?;

        Ok(())
    })
}
