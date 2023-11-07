use std::collections::HashSet;

use axum::http::HeaderMap;

pub mod fetcher;

lazy_static::lazy_static! {
    pub static ref CLIENT_HEADERS_KEEP: HashSet<String> = vec![
        "accept",
        "accept-encoding",
        "if-modified-since",
        "if-none-match",
        "range",
        "user-agent",
    ]
    .into_iter()
    .map(|x| x.to_string())
    .collect();
}

lazy_static::lazy_static! {
    pub static ref RESPONSE_HEADERS_KEEP: HashSet<String> = vec![
        "content-type",
        "content-length",
        "etag",
        "last-modified",
        "vary",
        "cache-control",
        "age",
        "accept-ranges",
        "transfer-encoding",
    ]
    .into_iter()
    .map(|x| x.to_string())
    .collect();
}

pub fn filter_headers(headers: &HeaderMap, keep: &HashSet<String>) -> HeaderMap {
    headers
        .iter()
        .filter(|(name, _value)| keep.contains(&name.to_string()))
        .fold(HeaderMap::new(), |mut acc, (name, value)| {
            acc.insert(name.clone(), value.clone());
            acc
        })
}
