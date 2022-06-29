// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use libtor::{Tor, TorFlag};

pub fn run_tor() {
    match Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/reverseproxy".into()))
        .flag(TorFlag::SocksPort(19054))
        .flag(TorFlag::Hush())
        .start()
    {
        Ok(r) => log::info!("tor exit result: {}", r),
        Err(e) => log::error!("tor error: {}", e),
    };
}
