// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::Result;
use futures::FutureExt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_socks::tcp::Socks5Stream;
use tokio_socks::IntoTargetAddr;

use crate::CONFIG;

lazy_static! {
    pub static ref TOR_PROXY_ADDR: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
}

struct TpcSocks5Stream;

impl TpcSocks5Stream {
    pub async fn connect<'a>(
        proxy: SocketAddr,
        dest: impl IntoTargetAddr<'a>,
    ) -> Result<TcpStream> {
        let sock = Socks5Stream::connect(proxy, dest).await?;
        Ok(sock.into_inner())
    }
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

async fn transfer(mut inbound: TcpStream) -> Result<()> {
    log::debug!("Connecting to {}", CONFIG.forward);

    let mut outbound: TcpStream = if CONFIG.use_tor {
        TpcSocks5Stream::connect(*TOR_PROXY_ADDR, CONFIG.forward.clone()).await?
    } else if let Some(proxy) = CONFIG.socks5_proxy {
        TpcSocks5Stream::connect(proxy, CONFIG.forward.clone()).await?
    } else {
        TcpStream::connect(CONFIG.forward.clone()).await?
    };

    log::info!("Connected to {}", CONFIG.forward);

    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        tokio::io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        tokio::io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    log::info!("Transfer closed");

    Ok(())
}
