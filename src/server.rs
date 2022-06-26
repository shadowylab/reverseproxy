// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

use crate::{forwarder::ReverseProxy, CONFIG};

lazy_static! {
    static ref REVERSE_PROXY: ReverseProxy = ReverseProxy::new(&CONFIG.forward, &CONFIG.proxy)
        .timeout(Duration::from_secs(CONFIG.timeout));
}

pub async fn run() -> std::io::Result<()> {
    let http_server =
        HttpServer::new(move || App::new().route("/{tail:.*}", web::to(generic_request)));

    match http_server.bind(CONFIG.server) {
        Ok(server) => {
            log::info!("Listening on {}", CONFIG.server);
            server.run().await
        }
        Err(error) => panic!("Impossible to start local server: {:?}", error),
    }
}

async fn generic_request(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    REVERSE_PROXY.forward(req, body).await
}
