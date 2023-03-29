#![allow(dead_code)]

use std::{net::SocketAddr, sync::Arc};

use color_eyre::Report;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::{rustls::ClientConfig, TlsConnector};
use tracing::info;
use tracing_subscriber::EnvFilter;
use webpki::DNSNameRef;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Report> {
    setup()?;

    let res = tokio::try_join!(fetch_url("first"), fetch_url("second"));
    info!(?res, "All done!");

    Ok(())
}

async fn fetch_url(name: &str) -> Result<&str, Report> {
    let addr: SocketAddr = ([1, 1, 1, 1], 443).into();
    let socket = TcpStream::connect(addr).await?;

    // establish a TLS session...
    let connector: TlsConnector = {
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        Arc::new(config).into()
    };

    let dnsname = DNSNameRef::try_from_ascii_str("one.one.one.one")?;
    let mut socket = connector.connect(dnsname, socket).await?;

    socket.write_all(b"GET / HTTP/1.1\r\n").await?;
    socket.write_all(b"Host: one.one.one.one\r\n").await?;
    socket.write_all(b"User-Agent: cool-bear\r\n").await?;
    socket.write_all(b"Connection: close\r\n").await?;
    socket.write_all(b"\r\n").await?;

    let mut response = String::with_capacity(256);
    socket.read_to_string(&mut response).await?;

    let status = response.lines().next().unwrap_or_default();
    info!(%status, %name, "Got response!");

    Ok(name)
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
