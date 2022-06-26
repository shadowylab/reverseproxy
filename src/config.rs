// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

pub use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Local server address and port
    #[clap(long("server"))]
    pub server: SocketAddr,
    /// Address to forward traffic (ex. torhiddenservice.onion:80)
    #[clap(long("forward"))]
    pub forward: String,
    /// Use Tor default Socks5 proxy
    #[clap(long("use-tor"), action = ArgAction::SetTrue)]
    pub use_tor: bool,
    /// Socks5 proxy (ex. 127.0.0.1:9050)
    #[clap(long("socks5-proxy"))]
    pub socks5_proxy: Option<SocketAddr>,
    #[clap(long("domain"))]
    pub domain: Option<String>,
}
