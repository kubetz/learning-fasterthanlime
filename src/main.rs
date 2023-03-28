#![allow(dead_code)]

use std::net::SocketAddr;

use color_eyre::Report;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

pub const URL_1: &str = "https://fasterthanli.me/articles/whats-in-the-box";
pub const URL_2: &str = "https://fasterthanli.me/series/advent-of-code-2020/part-13";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Report> {
    setup()?;

    let results = vec![fetch_url(URL_1), fetch_url(URL_2)]
        .into_iter()
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    for res in results {
        res?
    }

    Ok(())
}

async fn fetch_url(name: &str) -> Result<(), Report> {
    let addr: SocketAddr = ([1, 1, 1, 1], 80).into();
    let mut socket = TcpStream::connect(addr).await?;

    socket.write_all(b"GET / HTTP/1.1\r\n").await?;
    socket.write_all(b"Host: 1.1.1.1\r\n").await?;
    socket.write_all(b"User-Agent: cool-bear\r\n").await?;
    socket.write_all(b"Connection: close\r\n").await?;
    socket.write_all(b"\r\n").await?;

    let mut response = String::with_capacity(256);
    socket.read_to_string(&mut response).await?;

    let status = response.lines().next().unwrap_or_default();
    info!(%status, %name, "Got a response!");

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
