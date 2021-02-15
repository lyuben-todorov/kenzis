//! This example demonstrates an HTTP server that serves files from a directory.
//!
//! Checkout the `README.md` for guidance.

use std::{
    sync::Arc,
};

use anyhow::{Result};
use futures::{StreamExt, TryFutureExt};
use structopt::{self, StructOpt};
use kenzis::{Opt, fix_certs};
use std::net::SocketAddr;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;


fn server_addr() -> SocketAddr {
    "127.0.0.1:4433".parse::<SocketAddr>().unwrap()
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    run(opt)
}

#[tokio::main]
async fn run(options: Opt) -> Result<()> {
    info!("Starting KenzisRPC server!");
    let transport_config = quinn::TransportConfig::default();
    let mut server_config = quinn::ServerConfig::default();
    server_config.transport = Arc::new(transport_config);
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);

    if options.stateless_retry {
        server_config.use_stateless_retry(true);
    }

    fix_certs(&options, &mut server_config).unwrap();

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.listen(server_config.build());

    let (endpoint, mut incoming) = endpoint.bind(&server_addr())?;
    info!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = incoming.next().await {
        info!("connection incoming");
        tokio::spawn(
            handle_connection(conn).unwrap_or_else(move |e| {
                error!("connection failed: {reason}", reason = e.to_string())
            }),
        );
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Connecting) -> Result<()> {
    let quinn::NewConnection {
        connection,
        mut bi_streams,
        ..
    } = conn.await?;
    async {
        info!("established");

        // Each stream initiated by the client constitutes a new request.
        while let Some(Ok(stream)) = bi_streams.next().await {
            tokio::spawn(
                handle_request(stream)
                    .unwrap_or_else(move |e| error!("failed: {reason}", reason = e.to_string()))
            );
        }
        Ok(())
    }.await?;
    Ok(())
}

async fn handle_request(
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    info!("Req!");
    let bytes = recv.read_to_end(0).await.unwrap();
    let message = String::from_utf8(bytes).unwrap();
    println!("{}",message);
    Ok(())
}