// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

#[derive(Debug)]
pub enum Error {
    AddrParse(AddrParseError),
    MissingProtocol,
    UnsupportedProtocol,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddrParse(e) => write!(f, "{e}"),
            Self::MissingProtocol => write!(f, "Protocol not specified"),
            Self::UnsupportedProtocol => write!(f, "Unsupported protocol"),
        }
    }
}

impl From<AddrParseError> for Error {
    fn from(e: AddrParseError) -> Self {
        Self::AddrParse(e)
    }
}
