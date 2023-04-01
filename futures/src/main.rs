use std::{net::SocketAddr, sync::Arc};

use color_eyre::Report;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};
use tokio_rustls::TlsConnector;
use tracing::info;
use tracing_subscriber::EnvFilter;

// own try-join implementation
mod tj;

// running multiple futures in parallel while using single threaded runtime
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Report> {
    setup()?;

    // using our own try_join implementation
    let res = tj::try_join(fetch_1111("first"), fetch_1111("second")).await?;
    info!(?res, "All done!");

    Ok(())
}

async fn fetch_1111(name: &str) -> Result<&str, Report> {
    // create raw TCP connection for 1.1.1.1:443
    let addr: SocketAddr = ([1, 1, 1, 1], 443).into();
    let socket = TcpStream::connect(addr).await?;

    let connector: TlsConnector = {
        // build root certificate store from webpki-roots
        let mut root_store = RootCertStore::empty();
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        // build TLS client configuration
        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // create TLS connector
        Arc::new(config).into()
    };

    // connect to the one.one.one.one server using TLS by encapsulating the TCP socket
    let domain = ServerName::try_from("one.one.one.one")?;
    let mut socket = connector.connect(domain, socket).await?;

    // write GET request by using the TLS socket
    socket.write_all(b"GET / HTTP/1.1\r\n").await?;
    socket.write_all(b"Host: one.one.one.one\r\n").await?;
    socket.write_all(b"User-Agent: ferris\r\n").await?;
    socket.write_all(b"Connection: close\r\n").await?;
    socket.write_all(b"\r\n").await?;

    // read response from the TLS socket - we read the whole thing just to flex
    let mut buf = [0; 1024];
    let mut res = String::new();
    while socket.read(&mut buf).await.is_ok() {
        let string = std::str::from_utf8(&buf)?;
        res.push_str(string);
    }

    // get status of the response
    let status = res.lines().next().unwrap_or("No response");
    info!(%status, %name, "Got response!");

    Ok(name)
}

// setup tracing and color_eyre
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
