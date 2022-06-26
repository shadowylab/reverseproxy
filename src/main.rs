// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate lazy_static;

use anyhow::Result;

mod config;
mod logger;
mod tcp;

use config::{Args, Parser};
use logger::Logger;

lazy_static! {
    pub static ref CONFIG: Args = Args::parse();
}

#[tokio::main]
async fn main() -> Result<()> {
    Logger::init();

    tcp::run().await
}
