// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::sync::Arc;

use arti_client::config::BoolOrAuto;
use arti_client::StreamPrefs;
#[cfg(feature = "tor")]
use arti_client::{DataStream, TorClient};
use futures::FutureExt;
use tokio::io::{copy, split, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
#[cfg(feature = "tor")]
use tor_rtcompat::PreferredRuntime;

mod socks;

use self::socks::TpcSocks5Stream;
use crate::util::random_id;
use crate::Result;

enum Connection {
    TcpStream(TcpStream),
    #[cfg(feature = "tor")]
    DataStream(DataStream),
}

pub struct TcpReverseProxy {
    local_addr: SocketAddr,
    forward_addr: String,
    socks5_proxy: Option<SocketAddr>,
    #[cfg(feature = "tor")]
    tor: Option<TorClient<PreferredRuntime>>,
}

impl TcpReverseProxy {
    pub fn new(local_addr: SocketAddr, forward_addr: String) -> Self {
        Self {
            local_addr,
            forward_addr,
            socks5_proxy: None,
            #[cfg(feature = "tor")]
            tor: None,
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

        log::info!("Listening on {}", self.local_addr);

        let this = Arc::new(self);
        while let Ok((inbound, _)) = listener.accept().await {
            let transfer = this.clone().transfer(inbound).map(|r| {
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

    async fn connect(self: Arc<Self>) -> Result<Connection> {
        if let Some(proxy) = self.socks5_proxy {
            Ok(Connection::TcpStream(
                TpcSocks5Stream::connect(proxy, self.forward_addr.as_str()).await?,
            ))
        } else if let Some(tor) = &self.tor {
            let mut prefs = StreamPrefs::default();
            prefs.connect_to_onion_services(BoolOrAuto::Explicit(true));

            Ok(Connection::DataStream(
                tor.connect_with_prefs(self.forward_addr.as_str(), &prefs)
                    .await?,
            ))
        } else {
            Ok(Connection::TcpStream(
                TcpStream::connect(self.forward_addr.as_str()).await?,
            ))
        }
    }

    async fn transfer(self: Arc<Self>, inbound: TcpStream) -> Result<()> {
        let connection_id: String = random_id();

        log::debug!("Connecting to {}", self.forward_addr);

        let outbound: Connection = self.connect().await?;

        log::info!("Connection {} enstablished", connection_id);

        let (mut ri, mut wi) = split(inbound);

        match outbound {
            Connection::TcpStream(outbound) => {
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
            }
            #[cfg(feature = "tor")]
            Connection::DataStream(outbound) => {
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
            }
        }

        log::info!("Connection {} closed", connection_id);
        Ok(())
    }
}
