mod prelude;

use std::{
    net::{IpAddr, ToSocketAddrs},
    str::FromStr,
};

use anyhow::anyhow;
use prelude::StdResult;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "home-dns-refresh",
    about = "Updates an Azure DNS zone A record to the currently external IP address"
)]
struct CliOptions {
    #[structopt(short = "t", long = "tenant-id")]
    pub tenant_id: String,

    #[structopt(short = "c", long = "client-id")]
    pub client_id: String,

    #[structopt(short = "p", long = "client-secret")]
    pub client_secret: String,

    #[structopt(short = "s", long = "subscription-id")]
    pub subscription_id: String,

    #[structopt(short = "r", long = "resource-group")]
    pub resource_group_name: String,

    #[structopt(short = "z", long = "zone-name")]
    pub zone_name: String,

    #[structopt(short = "d", long = "sub-domain")]
    pub sub_domain: String,

    #[structopt(long = "ttl")]
    pub ttl: Option<i32>,
}

#[tokio::main]
async fn main() -> StdResult {
    let cli_options = CliOptions::from_args();

    let scope = "https://management.core.windows.net/.default";
    let record_type = "A";

    let hostname = format!("{}.{}", cli_options.sub_domain, cli_options.zone_name);
    let default_ttl = 60;

    // Get the current assigned IP.
    let assigned_ip = resolve_hostname_to_ip(hostname)?;
    println!("resolving IP: {}", assigned_ip);

    // Get the current broadband IP.
    let external_ip_string = reqwest::get("http://ifconfig.me/ip").await?.text().await?;
    let external_ip = IpAddr::from_str(external_ip_string.trim())?;
    println!("broadband IP: {}", external_ip);

    if external_ip == assigned_ip {
        println!("No work to do");
        return Ok(());
    }

    println!("IP updating");

    // Get an auth token.
    let azure_token = request_azure_token(
        cli_options.tenant_id,
        cli_options.client_id,
        cli_options.client_secret,
        scope,
    )
    .await?;

    update_azure_ip(
        azure_token,
        cli_options.subscription_id,
        cli_options.resource_group_name,
        cli_options.zone_name,
        record_type,
        cli_options.sub_domain,
        cli_options.ttl.unwrap_or(default_ttl),
        external_ip,
    )
    .await?;

    println!("IP updated");

    Ok(())
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AuthTokenReply {
    token_type: Option<String>,
    expires_in: Option<i64>,
    ext_expires_in: Option<i64>,
    access_token: String,
}

#[allow(clippy::too_many_arguments)]
async fn update_azure_ip(
    azure_token: impl Into<String>,
    subscription_id: impl Into<String>,
    resource_group_name: impl Into<String>,
    zone_name: impl Into<String>,
    record_type: impl Into<String>,
    sub_domain: impl Into<String>,
    ttl: impl Into<i32>,
    external_ip_string: impl Into<IpAddr>,
) -> StdResult {
    let http_client = reqwest::Client::new();

    let update_ip_url = format!(
        "https://management.azure.com/subscriptions/{}/resourceGroups/{}/providers/Microsoft.Network/dnsZones/{}/{}/{}?api-version=2018-05-01",
        subscription_id.into(),
        resource_group_name.into(),
        zone_name.into(),
        record_type.into(),
        sub_domain.into(),
    );

    let update_ip_request_body = json!(
        {
            "properties": {
                "TTL": ttl.into(),
                "ARecords": [
                    {
                        "ipv4Address": external_ip_string.into()
                    }
                ]
            }
        }
    );

    let update_ip_status = http_client
        .put(update_ip_url)
        .bearer_auth(azure_token.into())
        .json(&update_ip_request_body)
        .send()
        .await?
        .status();

    if update_ip_status != StatusCode::OK {
        return Err(anyhow!(format!(
            "bad status code return from azure ip update: (status_code={})",
            update_ip_status
        )));
    }

    Ok(())
}

async fn request_azure_token(
    tenant_id: impl Into<String>,
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    scope: impl Into<String>,
) -> StdResult<String> {
    let http_client = reqwest::Client::new();

    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        tenant_id.into()
    );

    let token_params = [
        ("client_id", client_id.into()),
        ("client_secret", client_secret.into()),
        ("scope", scope.into()),
        ("grant_type", "client_credentials".into()),
    ];

    let token_reply = http_client
        .post(token_url)
        .form(&token_params)
        .send()
        .await?
        .json::<AuthTokenReply>()
        .await?;

    let result = token_reply.access_token;

    Ok(result)
}

fn resolve_hostname_to_ip(hostname: impl Into<String>) -> StdResult<IpAddr> {
    let hostname = hostname.into();
    let hostname_socketaddr = format!("{}:0", hostname);

    let mut ip_addresses = hostname_socketaddr
        .to_socket_addrs()
        .map_err(|_| anyhow!(format!("Hostname {} could not be resolved", hostname)))?;

    let resolved_ip_address_option = ip_addresses.find(|s| s.is_ipv4());

    let Some(resolved_ip_address) = resolved_ip_address_option else {
        return Err(anyhow!(format!("Hostname {} resolved to no IPs", hostname_socketaddr)))?;
    };

    Ok(resolved_ip_address.ip())
}
