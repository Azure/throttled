use eyre::{Context, Result};
use serde::{Deserialize, Serialize};

pub async fn get_msi_token(client_id: &str) -> Result<OAuthResponse> {
    let res = reqwest::Client::new()
        .get("http://169.254.169.254/metadata/identity/oauth2/token")
        .header("Metadata", "true")
        .query(&[
            ("client_id", client_id),
            ("api-version", "2018-02-01"),
            ("resource", "https://management.azure.com/"),
        ])
        .send()
        .await
        .wrap_err_with(|| "failed to send token request request")?
        .text()
        .await
        .wrap_err_with(|| "failed to fetch oauth token response")?;

    let res: OAuthResponse = serde_json::from_str(&res[..])
        .wrap_err_with(|| format!("failed to deserialize oauth token response: {}", res))?;

    Ok(res)
}

pub async fn get_sp_token(
    client_id: &str,
    client_secret: &str,
    tenant_id: &str,
    resource: &str,
) -> Result<OAuthResponse> {
    let params = [
        (("client_id", client_id)),
        (("client_secret", client_secret)),
        (("grant_type", "client_credentials")),
        (("resource", resource)),
    ];

    let client = reqwest::Client::new();
    let req = client
        .post(&format!(
            "https://login.microsoftonline.com/{}/oauth2/token",
            tenant_id,
        ))
        .form(&params);

    let res = req
        .send()
        .await
        .wrap_err_with(|| "failed to send token request request")?
        .text()
        .await
        .wrap_err_with(|| "failed to fetch oauth token response")?;

    let res: OAuthResponse = serde_json::from_str(&res[..])
        .wrap_err_with(|| "failed to deserialize oauth token response")?;

    Ok(res)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: String,
    pub expires_on: String,
    pub ext_expires_in: Option<String>,
    pub not_before: String,
    pub resource: String,
    pub token_type: String,
}
