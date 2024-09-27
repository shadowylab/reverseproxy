// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(target_family = "unix")]
use tokio::net::UnixStream;

#[cfg(feature = "tor")]
use arti_client::DataStream;
use futures::FutureExt;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
#[cfg(target_family = "unix")]
use tokio::net::UnixListener;
use tokio::net::{TcpListener, TcpStream};

mod socks;

use crate::config::{ForwardAddr, ListenAddr};
#[cfg(feature = "tor")]
use crate::tor;
use crate::Result;

trait Connection: AsyncRead + AsyncWrite + Unpin + Send {}

impl Connection for TcpStream {}

#[cfg(feature = "tor")]
impl Connection for DataStream {}

#[cfg(target_family = "unix")]
impl Connection for UnixStream {}

pub struct ReverseProxy {
    local_addr: ListenAddr,
    forward_addr: ForwardAddr,
    socks5_proxy: Option<SocketAddr>,
    #[cfg(feature = "tor")]
    tor: bool,
}

impl ReverseProxy {
    pub fn new(local_addr: ListenAddr, forward_addr: ForwardAddr) -> Self {
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
        match &self.local_addr {
            ListenAddr::Tcp(local_addr) => {
                let listener = TcpListener::bind(local_addr).await?;
                let this = Arc::new(self);
                this.process_listener(|| async { listener.accept().await.map(|(s, _)| s) })
                    .await?;
            }
            //ListenAddr::Udp(..) => todo!(),
            #[cfg(target_family = "unix")]
            ListenAddr::Unix(path) => {
                let listener = UnixListener::bind(path)?;
                let this = Arc::new(self);
                this.process_listener(|| async { listener.accept().await.map(|(s, _)| s) })
                    .await?;
            }
        }

        Ok(())
    }

    async fn process_listener<F, S, Fut>(self: Arc<Self>, mut accept: F) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<S, io::Error>>,
        S: AsyncRead + AsyncWrite + Send + 'static,
    {
        println!("Listening on '{}'.", self.local_addr);
        println!(
            "Incoming connections will be forwarded to '{}'.",
            self.forward_addr
        );
        while let Ok(inbound) = accept().await {
            let transfer = self.clone().transfer(inbound).map(|r| {
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
        match &self.forward_addr {
            ForwardAddr::Tcp(forward_addr) => {
                #[cfg(feature = "tor")]
                if self.tor {
                    let client = tor::client().await?;
                    return Ok(Box::new(client.connect(forward_addr).await?));
                }

                if let Some(proxy) = self.socks5_proxy {
                    Ok(Box::new(
                        socks::connect(proxy, forward_addr.as_str()).await?,
                    ))
                } else {
                    Ok(Box::new(TcpStream::connect(forward_addr).await?))
                }
            }
            // ForwardAddr::Udp(_forward_addr) => {
            //     todo!()
            // }
            #[cfg(target_family = "unix")]
            ForwardAddr::Unix(path) => Ok(Box::new(UnixStream::connect(path).await?)),
        }
    }

    async fn transfer<T>(self: Arc<Self>, inbound: T) -> Result<()>
    where
        T: AsyncRead + AsyncWrite,
    {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpStream, UnixListener};

    use super::*;

    #[tokio::test]
    async fn test_tcp_to_unix_socket() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.sock");

        let local_addr = "127.0.0.1:8888".parse().unwrap();
        let listen_addr = ListenAddr::Tcp(local_addr);
        let forward_addr = ForwardAddr::Unix(path.clone());

        // Unix socket server
        tokio::spawn(async move {
            let listener = UnixListener::bind(path).unwrap();
            while let Ok((mut stream, ..)) = listener.accept().await {
                let mut buf = [0; 14];
                stream.read(&mut buf).await.unwrap();
                stream.write_all(&buf).await.unwrap();
            }
        });

        // TCP to Unix socket proxy
        let proxy = ReverseProxy::new(listen_addr, forward_addr);
        tokio::spawn(async move { proxy.run().await.unwrap() });

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Client sends data and expects to receive the same data back
        let msg = b"hello-from-tcp";
        let mut client = TcpStream::connect(local_addr).await.unwrap();
        client.write_all(msg).await.unwrap();
        let mut buf = vec![0; 14];
        let n = client.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], msg);
    }

    #[tokio::test]
    async fn test_unix_socket_to_tcp() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.sock");

        let addr = "127.0.0.1:8889";
        let listen_addr = ListenAddr::Unix(path.clone());
        let forward_addr = ForwardAddr::Tcp(addr.to_string());

        // Tcp server
        tokio::spawn(async move {
            let listener = TcpListener::bind(addr).await.unwrap();
            while let Ok((mut stream, ..)) = listener.accept().await {
                let mut buf = [0; 15];
                stream.read(&mut buf).await.unwrap();
                stream.write_all(&buf).await.unwrap();
            }
        });

        // Unix socket to TCP proxy
        let proxy = ReverseProxy::new(listen_addr, forward_addr);
        tokio::spawn(async move { proxy.run().await.unwrap() });

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Client sends data and expects to receive the same data back
        let msg = b"hello-from-unix";
        let mut client = UnixStream::connect(path).await.unwrap();
        client.write_all(msg).await.unwrap();
        let mut buf = vec![0; 15];
        let n = client.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], msg);
    }
}
