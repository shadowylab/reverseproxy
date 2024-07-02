// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use tracing::Level;

pub struct Logger;

impl Logger {
    pub fn init() {
        let level: Level = if cfg!(debug_assertions) {
            Level::DEBUG
        } else {
            Level::INFO
        };

        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(level)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }
}
