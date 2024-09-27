// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::sync::Arc;

#[cfg(feature = "tor")]
use arti_client::DataStream;
use futures::FutureExt;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod socks;

#[cfg(feature = "tor")]
use crate::tor;
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
    tor: bool,
}

impl TcpReverseProxy {
    pub fn new(local_addr: SocketAddr, forward_addr: String) -> Self {
        Self {
            local_addr,
            forward_addr,
            socks5_proxy: None,
            #[cfg(feature = "tor")]
            tor: false,
        }
    }

    pub fn socks5_proxy(self, socks5_proxy: Option<SocketAddr>) -> Self {
        Self {
            socks5_proxy,
            ..self
        }
    }

    #[cfg(feature = "tor")]
    pub fn tor(mut self, enable: bool) -> Self {
        self.tor = enable;
        self
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.local_addr).await?;

        println!("Listening on {}", self.local_addr);

        let this = Arc::new(self);
        while let Ok((inbound, _)) = listener.accept().await {
            let transfer = this.clone().transfer(inbound).map(|r| {
                if let Err(err) = r {
                    eprintln!("{err}");

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
        if self.tor {
            let client = tor::client().await?;
            return Ok(Box::new(client.connect(self.forward_addr.as_str()).await?));
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
        let outbound: Box<dyn Connection> = self.connect().await?;

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

        Ok(())
    }
}
