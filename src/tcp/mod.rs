// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::Result;
use futures::FutureExt;
use tokio::io::{copy, split, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod socks;

use socks::TpcSocks5Stream;

use crate::{util::random_id, CONFIG};

lazy_static! {
    pub static ref DEFAULT_TOR_SOCKS5_ADDR: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
}

pub async fn run() -> Result<()> {
    let listener = TcpListener::bind(CONFIG.server).await?;

    log::info!("Listening on {}", CONFIG.server);

    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound).map(|r| {
            if let Err(err) = r {
                log::error!("Transfer failed: {}", err);
            }
        });

        tokio::spawn(transfer);
    }

    Ok(())
}

async fn connect() -> Result<TcpStream> {
    if CONFIG.use_tor {
        TpcSocks5Stream::connect(*DEFAULT_TOR_SOCKS5_ADDR, CONFIG.forward.clone()).await
    } else if let Some(proxy) = CONFIG.socks5_proxy {
        TpcSocks5Stream::connect(proxy, CONFIG.forward.clone()).await
    } else {
        Ok(TcpStream::connect(CONFIG.forward.clone()).await?)
    }
}

async fn transfer(inbound: TcpStream) -> Result<()> {
    let connection_id: String = random_id();

    log::debug!("Connecting to {}", CONFIG.forward);

    let outbound: TcpStream = connect().await?;

    log::info!("Connection {} enstablished", connection_id);

    let (mut ri, mut wi) = split(inbound);
    let (mut ro, mut wo) = split(outbound);

    let client_to_server = async {
        copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    log::info!("Connection {} closed", connection_id);
    Ok(())
}
