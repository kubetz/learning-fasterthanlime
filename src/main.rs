#![allow(dead_code)]

use color_eyre::Report;
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub const URL_1: &str = "https://fasterthanli.me/articles/whats-in-the-box";
pub const URL_2: &str = "https://fasterthanli.me/series/advent-of-code-2020/part-13";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Report> {
    setup()?;

    let client = Client::new();

    let results = vec![fetch_url(client.clone(), URL_1), fetch_url(client, URL_2)]
        .into_iter()
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    for res in results {
        res?
    }

    Ok(())
}

async fn fetch_url(client: Client, url: &str) -> Result<(), Report> {
    let res = client.get(url).send().await?.error_for_status()?;
    info!(%url, content_type = ?res.headers().get("content-type"), "Got a response!");
    Ok(())
}

fn type_name_of<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
