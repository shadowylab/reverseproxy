// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::io::Write;

use env_logger::{
    fmt::{Style, Timestamp},
    Builder, Env,
};
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

        Builder::from_env(Env::default().default_filter_or(log_level.as_str()))
            .format(move |buf, record| {
                let level: Level = record.level();

                let timestamp: Timestamp = buf.timestamp();
                let level_style: Style = buf.default_level_style(level);

                writeln!(
                    buf,
                    "[{} {} {}] {}",
                    timestamp,
                    level_style.value(level),
                    record.target(),
                    record.args()
                )
            })
            .init();
    }
}
