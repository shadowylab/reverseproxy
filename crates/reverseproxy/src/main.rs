// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use anyhow::Result;
use reverseproxy_config::{Args, Parser};
use reverseproxy_logger::Logger;
use reverseproxy_tcp::TcpReverseProxy;
use reverseproxy_tor::run_tor;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    Logger::init();

    if args.use_tor {
        tokio::task::spawn_blocking(run_tor);
    }

    let socks5_proxy: Option<SocketAddr> = if args.use_tor {
        Some(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::LOCALHOST,
            19054,
        )))
    } else {
        args.socks5_proxy
    };

    TcpReverseProxy::new(args.local_addr, args.forward_addr, socks5_proxy)
        .run()
        .await
}
