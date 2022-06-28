// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;

mod config;
mod logger;
mod tcp;
mod util;

use config::{Args, Parser};
use logger::Logger;
use tcp::TcpReverseProxy;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    Logger::init();

    TcpReverseProxy::new(
        args.local_addr,
        args.forward_addr,
        args.socks5_proxy,
        args.use_tor,
    )
    .run()
    .await
}
