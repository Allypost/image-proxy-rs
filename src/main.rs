#![allow(clippy::pedantic)]

mod config;
pub mod log;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match dotenvy::dotenv().err() {
        Some(e) if e.not_found() => println!("No .env file found"),
        Some(e) => return Err(e.into()),
        None => {}
    }
    log::init();

    log::debug!(config = ?*config::CONFIG);

    server::run().await
}
