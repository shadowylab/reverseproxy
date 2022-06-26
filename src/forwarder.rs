// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;
use std::time::Duration;

use actix_web::{web::Bytes, HttpRequest, HttpResponse, HttpResponseBuilder};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Method, RequestBuilder, Response,
};

lazy_static! {
    static ref HEADER_X_FORWARDED_FOR: HeaderName =
        HeaderName::from_lowercase(b"x-forwarded-for").unwrap();
    static ref HOP_BY_HOP_HEADERS: Vec<HeaderName> = vec![
        HeaderName::from_lowercase(b"connection").unwrap(),
        HeaderName::from_lowercase(b"proxy-connection").unwrap(),
        HeaderName::from_lowercase(b"keep-alive").unwrap(),
        HeaderName::from_lowercase(b"proxy-authenticate").unwrap(),
        HeaderName::from_lowercase(b"proxy-authorization").unwrap(),
        HeaderName::from_lowercase(b"te").unwrap(),
        HeaderName::from_lowercase(b"trailer").unwrap(),
        HeaderName::from_lowercase(b"transfer-encoding").unwrap(),
        HeaderName::from_lowercase(b"upgrade").unwrap(),
    ];
    static ref HEADER_TE: HeaderName = HeaderName::from_lowercase(b"te").unwrap();
    static ref HEADER_CONNECTION: HeaderName = HeaderName::from_lowercase(b"connection").unwrap();
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct ReverseProxy {
    forward_url: String,
    timeout: Duration,
    client: reqwest::Client,
}

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
}

impl ReverseProxy {
    pub fn new(forward_url: &String, proxy: &Option<String>) -> Self {
        let mut client = reqwest::Client::builder();

        if let Some(proxy) = proxy {
            client = client.proxy(reqwest::Proxy::all(proxy).unwrap());
        }

        Self {
            forward_url: forward_url.into(),
            timeout: DEFAULT_TIMEOUT,
            client: client.build().unwrap(),
        }
    }

    pub fn timeout(mut self, duration: Duration) -> ReverseProxy {
        self.timeout = duration;
        self
    }

    fn x_forwarded_for_value(&self, req: &HttpRequest) -> String {
        let mut result = String::new();

        for (key, value) in req.headers() {
            if key == *HEADER_X_FORWARDED_FOR {
                result.push_str(value.to_str().unwrap());
                break;
            }
        }

        // adds client IP address
        // to x-forwarded-for header
        // if it's available
        if let Some(peer_addr) = req.peer_addr() {
            add_client_ip(&mut result, peer_addr);
        }

        result
    }

    fn forward_uri(&self, req: &HttpRequest) -> String {
        let forward_uri = match req.uri().query() {
            Some(query) => format!("{}{}?{}", self.forward_url, req.uri().path(), query),
            None => format!("{}{}", self.forward_url, req.uri().path()),
        };

        forward_uri
    }

    pub async fn forward(&self, req: HttpRequest, body: Bytes) -> HttpResponse {
        let forward_uri: String = self.forward_uri(&req);
        let forward_method: Method = req.method().clone();
        let mut forward_headers = HeaderMap::new();

        log::info!("Forward uri: {}", forward_uri);

        req.headers().iter().for_each(|(name, value)| {
            forward_headers.insert(name, value.clone());
        });

        log::debug!("Removing connection header...");
        remove_connection_headers(&mut forward_headers);
        log::debug!("Removing request hop by hop");
        remove_request_hop_by_hop_headers(&mut forward_headers);

        log::debug!("Building forward request");
        let forward_req: RequestBuilder = self
            .client
            .request(forward_method, forward_uri.as_str())
            .headers(forward_headers)
            .header(&(*HEADER_X_FORWARDED_FOR), self.x_forwarded_for_value(&req))
            .body(body)
            .timeout(self.timeout);

        log::debug!("Getting forward response...");
        let forward_res: Response = forward_req.send().await.unwrap();

        log::debug!("Building client response...");
        let mut back_res: HttpResponseBuilder = HttpResponse::build(forward_res.status());

        // copy headers
        for (key, value) in forward_res.headers() {
            if !HOP_BY_HOP_HEADERS.contains(key) {
                back_res.append_header((key.clone(), value.clone()));
            }
        }

        log::info!("Sending response to client");
        back_res.body(forward_res.bytes().await.unwrap())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

fn add_client_ip(fwd_header_value: &mut String, client_addr: SocketAddr) {
    if !fwd_header_value.is_empty() {
        fwd_header_value.push_str(", ");
    }

    let client_ip_str = &format!("{}", client_addr.ip());
    fwd_header_value.push_str(client_ip_str);
}

fn remove_connection_headers(headers: &mut HeaderMap) {
    let mut headers_to_delete: Vec<String> = Vec::new();
    let header_connection = &(*HEADER_CONNECTION);

    if headers.contains_key(header_connection) {
        if let Some(connection_header_value) = headers.get(header_connection) {
            if let Ok(connection_header_value) = connection_header_value.to_str() {
                for h in connection_header_value.split(',').map(|s| s.trim()) {
                    headers_to_delete.push(String::from(h));
                }
            }
        }
    }

    for h in headers_to_delete {
        headers.remove(h);
    }
}

fn remove_request_hop_by_hop_headers(headers: &mut HeaderMap) {
    for h in HOP_BY_HOP_HEADERS.iter() {
        if headers.contains_key(h)
            && (headers[h].is_empty() || (h == *HEADER_TE && headers[h] == "trailers"))
        {
            continue;
        }
        headers.remove(h);
    }
}