// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

mod config;
mod tcp;
#[cfg(feature = "tor")]
mod tor;

use self::config::{Args, Parser};
use self::tcp::TcpReverseProxy;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    #[allow(unused_mut)]
    let mut reverse_proxy = TcpReverseProxy::new(args.local_addr, args.forward_addr);

    #[cfg(feature = "tor")]
    {
        reverse_proxy = reverse_proxy.tor(args.tor);
    }

    reverse_proxy.socks5_proxy(args.socks5_proxy).run().await
}
