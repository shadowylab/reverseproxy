// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
#[cfg(feature = "tor")]
use std::net::{Ipv4Addr, SocketAddrV4};

mod config;
mod logger;
mod tcp;
#[cfg(feature = "tor")]
mod tor;
mod util;

use self::config::{Args, Parser};
use self::logger::Logger;
use self::tcp::TcpReverseProxy;
#[cfg(feature = "tor")]
use self::tor::Tor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();
    Logger::init();

    #[cfg(feature = "tor")]
    if args.use_tor {
        let tor = Tor::new("/tmp/reverseproxy".into(), 19054);
        tokio::task::spawn_blocking(move || tor.start());
    }

    #[cfg(feature = "tor")]
    let socks5_proxy: Option<SocketAddr> = if args.use_tor {
        Some(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::LOCALHOST,
            19054,
        )))
    } else {
        args.socks5_proxy
    };

    #[cfg(not(feature = "tor"))]
    let socks5_proxy: Option<SocketAddr> = args.socks5_proxy;

    TcpReverseProxy::new(args.local_addr, args.forward_addr, socks5_proxy)
        .run()
        .await
}
