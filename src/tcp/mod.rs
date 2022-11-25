// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use futures::FutureExt;
use tokio::io::{copy, split, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod socks;

use self::socks::TpcSocks5Stream;
use crate::util::random_id;

pub struct TcpReverseProxy {
    local_addr: SocketAddr,
    forward_addr: String,
    socks5_proxy: Option<SocketAddr>,
}

impl TcpReverseProxy {
    pub fn new(
        local_addr: SocketAddr,
        forward_addr: String,
        socks5_proxy: Option<SocketAddr>,
    ) -> Arc<Self> {
        Arc::new(Self {
            local_addr,
            forward_addr,
            socks5_proxy,
        })
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        let listener = TcpListener::bind(self.local_addr).await?;

        log::info!("Listening on {}", self.local_addr);

        while let Ok((inbound, _)) = listener.accept().await {
            let transfer = self.clone().transfer(inbound).map(|r| {
                if let Err(err) = r {
                    log::error!("Transfer failed: {}", err);

                    use tokio_socks::Error;
                    if let Some(Error::TtlExpired) = err.downcast_ref::<Error>() {
                        std::process::exit(0x1);
                    }
                }
            });

            tokio::spawn(transfer);
        }

        Ok(())
    }

    async fn connect(self: Arc<Self>) -> Result<TcpStream> {
        if let Some(proxy) = self.socks5_proxy {
            TpcSocks5Stream::connect(proxy, self.forward_addr.as_str()).await
        } else {
            Ok(TcpStream::connect(self.forward_addr.as_str()).await?)
        }
    }

    async fn transfer(self: Arc<Self>, inbound: TcpStream) -> Result<()> {
        let connection_id: String = random_id();

        log::debug!("Connecting to {}", self.forward_addr);

        let outbound: TcpStream = self.connect().await?;

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
}
