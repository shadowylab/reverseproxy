// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

pub use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Local server address and port
    #[clap(long("server"))]
    pub server: SocketAddr,
    /// Address to forward traffic (ex. http://torhiddenservice.onion)
    #[clap(long("forward"))]
    pub forward: String,
    /// Proxy (ex. Tor proxy: socks5h://127.0.0.1:9050)
    #[clap(long("proxy"))]
    pub proxy: Option<String>,
    /// Request timeout in seconds
    #[clap(short('t'), long("timeout"), default_value_t = 60)]
    pub timeout: u64,
}
