// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use arti_client::config::TorClientConfigBuilder;
use arti_client::{TorClient, TorClientConfig};
use tokio::sync::OnceCell;
use tor_rtcompat::PreferredRuntime;

use super::Result;

static TOR_CLIENT: OnceCell<TorClient<PreferredRuntime>> = OnceCell::const_new();

async fn init_client() -> Result<TorClient<PreferredRuntime>> {
    // Compose config
    let mut config = TorClientConfigBuilder::default();
    config.address_filter().allow_onion_addrs(true);
    let config: TorClientConfig = config.build()?;

    println!("Bootstrapping tor...");

    let client = TorClient::builder()
        .config(config)
        .create_bootstrapped()
        .await?;

    println!("Tor bootstrap completed!");

    Ok(client)
}

/// Get or init tor client
#[inline]
pub async fn client<'a>() -> Result<&'a TorClient<PreferredRuntime>> {
    TOR_CLIENT
        .get_or_try_init(|| async { init_client().await })
        .await
}
