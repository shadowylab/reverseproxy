// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use tokio_socks::IntoTargetAddr;

use crate::Result;

pub struct TpcSocks5Stream;

impl TpcSocks5Stream {
    pub async fn connect<'a>(
        proxy: SocketAddr,
        dest: impl IntoTargetAddr<'a>,
    ) -> Result<TcpStream> {
        Ok(Socks5Stream::connect(proxy, dest).await?.into_inner())
    }
}
