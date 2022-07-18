// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use libtor::{Tor as LibTor, TorFlag};

pub struct Tor {
    data_dir: String,
    socks_port: u16,
}

impl Tor {
    pub fn new(data_dir: String, socks_port: u16) -> Self {
        Self {
            data_dir,
            socks_port,
        }
    }

    pub fn start(&self) {
        match LibTor::new()
            .flag(TorFlag::DataDirectory(self.data_dir.clone()))
            .flag(TorFlag::SocksPort(self.socks_port))
            //.flag(TorFlag::Hush())
            .start()
        {
            Ok(_) => {
                log::info!("Tor exited");
                std::process::exit(0x1);
            }
            Err(e) => log::error!("Tor error: {}", e),
        };
    }
}
