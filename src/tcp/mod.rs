// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

#[cfg(feature = "tor")]
use arti_client::config::BoolOrAuto;
#[cfg(feature = "tor")]
use arti_client::StreamPrefs;
#[cfg(feature = "tor")]
use arti_client::{DataStream, TorClient};
use futures::FutureExt;
use tokio::io::{copy, split, AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
#[cfg(feature = "tor")]
use tor_rtcompat::PreferredRuntime;

mod socks;

use self::socks::TpcSocks5Stream;
use crate::Result;

enum Connection {
    TcpStream(TcpStream),
    #[cfg(feature = "tor")]
    DataStream(DataStream),
}

impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::TcpStream(stream) => AsyncRead::poll_read(Pin::new(stream), cx, buf),
            #[cfg(feature = "tor")]
            Self::DataStream(stream) => AsyncRead::poll_read(Pin::new(stream), cx, buf),
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::TcpStream(stream) => AsyncWrite::poll_write(Pin::new(stream), cx, buf),
            #[cfg(feature = "tor")]
            Self::DataStream(stream) => AsyncWrite::poll_write(Pin::new(stream), cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::TcpStream(stream) => AsyncWrite::poll_flush(Pin::new(stream), cx),
            #[cfg(feature = "tor")]
            Self::DataStream(stream) => AsyncWrite::poll_flush(Pin::new(stream), cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::TcpStream(stream) => AsyncWrite::poll_shutdown(Pin::new(stream), cx),
            #[cfg(feature = "tor")]
            Self::DataStream(stream) => AsyncWrite::poll_shutdown(Pin::new(stream), cx),
        }
    }
}

impl From<TcpStream> for Connection {
    fn from(stream: TcpStream) -> Self {
        Self::TcpStream(stream)
    }
}

#[cfg(feature = "tor")]
impl From<DataStream> for Connection {
    fn from(stream: DataStream) -> Self {
        Self::DataStream(stream)
    }
}

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

    async fn connect(self: Arc<Self>) -> Result<Connection> {
        #[cfg(feature = "tor")]
        if let Some(tor) = &self.tor {
            let mut prefs = StreamPrefs::default();
            prefs.connect_to_onion_services(BoolOrAuto::Explicit(true));

            return Ok(tor
                .connect_with_prefs(self.forward_addr.as_str(), &prefs)
                .await?
                .into());
        }

        if let Some(proxy) = self.socks5_proxy {
            Ok(TpcSocks5Stream::connect(proxy, self.forward_addr.as_str())
                .await?
                .into())
        } else {
            Ok(TcpStream::connect(self.forward_addr.as_str()).await?.into())
        }
    }

    async fn transfer(self: Arc<Self>, inbound: TcpStream) -> Result<()> {
        let connection_id: usize = self.counter.fetch_add(1, Ordering::SeqCst);

        tracing::debug!("Connecting to {}", self.forward_addr);

        let outbound: Connection = self.connect().await?;

        tracing::info!("Connection #{connection_id} established");

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

        tracing::info!("Connection #{connection_id} closed");

        Ok(())
    }
}
