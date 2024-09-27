// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::SocketAddr;
#[cfg(target_family = "unix")]
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(feature = "tor")]
use clap::ArgAction;
pub use clap::Parser;

use crate::error::Error;

#[derive(Debug, Clone)]
pub enum ListenAddr {
    Tcp(SocketAddr),
    //Udp(SocketAddr),
    #[cfg(target_family = "unix")]
    Unix(PathBuf),
}

impl fmt::Display for ListenAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp(addr) => write!(f, "tcp://{addr}"),
            //Self::Udp(addr) => write!(f, "udp://{addr}"),
            #[cfg(target_family = "unix")]
            Self::Unix(path) => write!(f, "unix://{}", path.display()),
        }
    }
}

impl FromStr for ListenAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("://") {
            Some(("tcp", addr)) => Ok(Self::Tcp(addr.parse()?)),
            //Some(("udp", addr)) => Ok(Self::Udp(addr.parse()?)),
            #[cfg(target_family = "unix")]
            Some(("unix", path)) => Ok(Self::Unix(PathBuf::from(path))),
            Some((_, _)) => Err(Error::UnsupportedProtocol),
            None => Err(Error::MissingProtocol),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ForwardAddr {
    Tcp(String),
    //Udp(String),
    #[cfg(target_family = "unix")]
    Unix(PathBuf),
}

impl fmt::Display for ForwardAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp(addr) => write!(f, "tcp://{addr}"),
            //Self::Udp(addr) => write!(f, "udp://{addr}"),
            #[cfg(target_family = "unix")]
            Self::Unix(path) => write!(f, "unix://{}", path.display()),
        }
    }
}

impl FromStr for ForwardAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("://") {
            Some(("tcp", addr)) => Ok(Self::Tcp(addr.to_string())),
            //Some(("udp", addr)) => Ok(Self::Udp(addr.to_string())),
            #[cfg(target_family = "unix")]
            Some(("unix", path)) => Ok(Self::Unix(PathBuf::from(path))),
            Some((_, _)) => Err(Error::UnsupportedProtocol),
            None => Err(Error::MissingProtocol),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Local address and port (ex. tcp://127.0.0.1:8080, unix:///tmp/temp.sock)
    #[clap(name("LOCAL_ADDR"))]
    pub local_addr: ListenAddr,
    /// Address and port where to forward traffic (ex. tcp://torhiddenservice.onion:80, unix:///tmp/temp.sock)
    #[clap(name("FORWARD_ADDR"))]
    pub forward_addr: ForwardAddr,
    /// Socks5 proxy (ex. 127.0.0.1:9050, supported only with TCP forward addresses)
    #[clap(long("socks5-proxy"))]
    pub socks5_proxy: Option<SocketAddr>,
    /// Use embedded Tor client (supported only with TCP forward addresses)
    #[cfg(feature = "tor")]
    #[clap(long("tor"), action = ArgAction::SetTrue)]
    pub tor: bool,
}
