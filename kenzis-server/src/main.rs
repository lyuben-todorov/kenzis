//! This example demonstrates an HTTP server that serves files from a directory.
//!
//! Checkout the `README.md` for guidance.

use std::{
    sync::Arc,
};

use anyhow::{Result};
use futures::{StreamExt, TryFutureExt};
use structopt::{self, StructOpt};
use kenzis::{Opt, fix_certs, ClientSession};
use std::net::SocketAddr;
use std::sync::RwLock;

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
    let mut connection = conn.await.unwrap();
    let state = ClientSession::new_context();
    info!("Established session");

    while let Some(Ok((sent, recv))) =  connection.bi_streams.next().await {
        // Because it is a bidirectional stream, we can both sent and recieve.
        tokio::spawn(
            handle_request((sent,recv), state.clone())
                .unwrap_or_else(move |e| error!("failed: {reason}", reason = e.to_string()))
        );
    }

    Ok(())
}

async fn handle_request(
    (mut send, recv): (quinn::SendStream, quinn::RecvStream),
    session: Arc<RwLock<ClientSession>>
) -> Result<()> {
    info!("client opened stream");


    let bytes = recv.read_to_end(50).await.unwrap();
    let message = String::from_utf8(bytes).unwrap();
    println!("{}",message);
    Ok(())
}