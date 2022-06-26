// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

#[macro_use]
extern crate lazy_static;

mod config;
mod forwarder;
mod logger;
mod server;

use config::{Args, Parser};
use logger::Logger;

lazy_static! {
    pub static ref CONFIG: Args = Args::parse();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Logger::init();

    server::run().await
}
