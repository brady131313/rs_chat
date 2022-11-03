use std::{error::Error, net::SocketAddr};

use clap::Parser;
use server::server::Server;
use tracing::Level;

#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
/// Run an irc server
struct Args {
    #[arg(short, default_value = "127.0.0.1:4000")]
    /// The address to accept connections on
    address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let server = Server::bind(args.address).await?;

    tokio::select! {
        _ = server.listen() => {},
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down");
        }
    }

    Ok(())
}
