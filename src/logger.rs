// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use env_logger::{Builder, Env};
use log::Level;

pub struct Logger;

const DEFAULT_LOG_LEVEL: Level = Level::Info;

impl Logger {
    pub fn init() {
        let log_level: Level = if cfg!(debug_assertions) {
            Level::Debug
        } else {
            DEFAULT_LOG_LEVEL
        };

        Builder::from_env(Env::default().default_filter_or(log_level.as_str())).init();
    }
}
