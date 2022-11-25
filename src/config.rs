// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

pub use clap::{ArgAction, Parser};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Local address and port (ex. 127.0.0.1:8080)
    #[clap(name("LOCAL_ADDR"))]
    pub local_addr: SocketAddr,
    /// Address and port to forward traffic (ex. torhiddenservice.onion:80)
    #[clap(name("FORWARD_ADDR"))]
    pub forward_addr: String,
    /// Socks5 proxy (ex. 127.0.0.1:9050)
    #[clap(long("socks5-proxy"))]
    pub socks5_proxy: Option<SocketAddr>,
    /// Use embedded Tor client
    #[cfg(feature = "tor")]
    #[clap(long("use-tor"), action = ArgAction::SetTrue)]
    pub use_tor: bool,
}
