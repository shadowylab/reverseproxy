// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(feature = "tor")]
use arti_client::{DataStream, TorClient};
use futures::FutureExt;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
#[cfg(feature = "tor")]
use tor_rtcompat::PreferredRuntime;

mod socks;

use crate::Result;

trait Connection: AsyncRead + AsyncWrite + Unpin + Send {}

impl Connection for TcpStream {}

#[cfg(feature = "tor")]
impl Connection for DataStream {}

pub struct TcpReverseProxy {
    local_addr: SocketAddr,
    forward_addr: String,
    socks5_proxy: Option<SocketAddr>,
    #[cfg(feature = "tor")]
    tor: Option<TorClient<PreferredRuntime>>,
    counter: Arc<AtomicUsize>,
}

impl TcpReverseProxy {
    pub fn new(local_addr: SocketAddr, forward_addr: String) -> Self {
        Self {
            local_addr,
            forward_addr,
            socks5_proxy: None,
            #[cfg(feature = "tor")]
            tor: None,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn socks5_proxy(self, socks5_proxy: Option<SocketAddr>) -> Self {
        Self {
            socks5_proxy,
            ..self
        }
    }

    #[cfg(feature = "tor")]
    pub fn tor(self, client: TorClient<PreferredRuntime>) -> Self {
        Self {
            tor: Some(client),
            ..self
        }
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.local_addr).await?;

        tracing::info!("Listening on {}", self.local_addr);

        let this = Arc::new(self);
        while let Ok((inbound, _)) = listener.accept().await {
            let transfer = this.clone().transfer(inbound).map(|r| {
                if let Err(err) = r {
                    tracing::error!("Transfer failed: {}", err);

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

    async fn connect(self: Arc<Self>) -> Result<Box<dyn Connection>> {
        #[cfg(feature = "tor")]
        if let Some(tor) = &self.tor {
            return Ok(Box::new(tor.connect(self.forward_addr.as_str()).await?));
        }

        if let Some(proxy) = self.socks5_proxy {
            Ok(Box::new(
                socks::connect(proxy, self.forward_addr.as_str()).await?,
            ))
        } else {
            Ok(Box::new(
                TcpStream::connect(self.forward_addr.as_str()).await?,
            ))
        }
    }

    async fn transfer(self: Arc<Self>, inbound: TcpStream) -> Result<()> {
        let connection_id: usize = self.counter.fetch_add(1, Ordering::SeqCst);

        tracing::debug!("Connecting to {}", self.forward_addr);

        let outbound: Box<dyn Connection> = self.connect().await?;

        tracing::info!("Connection #{connection_id} established");

        let (mut ri, mut wi) = io::split(inbound);
        let (mut ro, mut wo) = io::split(outbound);

        let client_to_server = async {
            io::copy(&mut ri, &mut wo).await?;
            wo.shutdown().await
        };

        let server_to_client = async {
            io::copy(&mut ro, &mut wi).await?;
            wi.shutdown().await
        };

        tokio::try_join!(client_to_server, server_to_client)?;

        tracing::info!("Connection #{connection_id} closed");

        Ok(())
    }
}
