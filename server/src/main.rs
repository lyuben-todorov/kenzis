//! This example demonstrates an HTTP server that serves files from a directory.
//!
//! Checkout the `README.md` for guidance.

use std::{
    fs, io,
    net::SocketAddr,
    path::{PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use futures::{StreamExt, TryFutureExt};
use structopt::{self, StructOpt};
use kenzis::{Opt, ALPN_QUIC_HTTP, fix_certs};

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

mod common;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    run(opt)
}

#[tokio::main]
#[allow(clippy::field_reassign_with_default)] // https://github.com/rust-lang/rust-clippy/issues/6527
async fn run(options: Opt) -> Result<()> {
    info!("Starting KenzisRPC server!");
    let mut transport_config = quinn::TransportConfig::default();
    let mut server_config = quinn::ServerConfig::default();
    server_config.transport = Arc::new(transport_config);
    let mut server_config = quinn::ServerConfigBuilder::new(server_config);
    server_config.protocols(ALPN_QUIC_HTTP);

    if options.stateless_retry {
        server_config.use_stateless_retry(true);
    }

    fix_certs(&options, &mut server_config);

    let mut endpoint = quinn::Endpoint::builder();
    endpoint.listen(server_config.build());

    let (endpoint, mut incoming) = endpoint.bind(&options.listen)?;
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
    
    Ok(())
}

