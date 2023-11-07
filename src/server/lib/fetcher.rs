use std::time::Duration;

use axum::http::HeaderMap;
use scraper::{Html, Selector};
use url::Url;

use crate::log::trace;

#[tracing::instrument(skip(headers))]
pub async fn fetch_image_url(page_url: &str, headers: HeaderMap) -> anyhow::Result<String> {
    trace!(page_url = ?page_url, "fetching image url");

    let url = parse_url(page_url).ok_or(anyhow::anyhow!("invalid url"))?;
    let url = transform_url(url);

    trace!(url = %url, "transformed url");

    let html = fetch_html(url.as_ref(), headers).await?;

    trace!("got page html");

    let image_url = tokio::task::spawn_blocking(move || get_image_from_html(&html))
        .await?
        .and_then(|x| url.join(&x).ok());

    trace!(image_url = ?image_url, "got image url from page html");

    image_url
        .map(|x| x.to_string())
        .ok_or_else(|| anyhow::anyhow!("no image found in {}", page_url))
}

fn get_image_from_html(html: &str) -> Option<String> {
    let fragment = Html::parse_document(html);
    trace!("parsed page html");
    lazy_static::lazy_static! {
        static ref OG_IMAGE_SELECTOR: Selector = Selector::parse("meta[property=\"og:image\"]").unwrap();
        static ref IMG_SELECTOR: Selector = Selector::parse("img").unwrap();
    }

    fragment
        .select(&OG_IMAGE_SELECTOR)
        .next()
        .and_then(|x| x.value().attr("content"))
        .or_else(|| {
            fragment
                .select(&IMG_SELECTOR)
                .next()
                .and_then(|x| x.attr("src"))
        })
        .map(|x| x.to_string())
}

async fn fetch_html(url: &str, headers: HeaderMap) -> anyhow::Result<String> {
    trace!(url = ?url, "fetching html from url");
    reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()?
        .get(url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!(e))?
        .text()
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

fn parse_url(raw_url: &str) -> Option<Url> {
    let parsed = Url::parse(raw_url).ok()?;

    let scheme = parsed.scheme();

    if scheme != "http" && scheme != "https" {
        return None;
    }

    if parsed.cannot_be_a_base() {
        return None;
    }

    Some(parsed)
}

fn transform_url(url: Url) -> Url {
    match url.host().unwrap().to_string().as_str() {
        "imagexxx.host" => url.join("./big.html").unwrap(),
        _ => url,
    }
}
