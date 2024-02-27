pub mod analysis;
pub mod backend;
pub mod capabilities;
pub mod config;
pub mod diagnostics;
pub mod reference;
pub mod utils;

use backend::Backend;
use clap::Parser;
use tokio::net::TcpListener;
use tower_lsp::{LspService, Server};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "LMNtal Language Server")]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Port number to listen for incoming connections")]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let arg = Args::parse();

    env_logger::init();

    if let Some(port) = arg.port {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).await.unwrap();
        let (stream, _) = listener.accept().await.unwrap();

        let (read, write) = tokio::io::split(stream);
        #[cfg(feature = "runtime-agnostic")]
        let (read, write) = (read.compat(), write.compat_write());

        let (service, socket) = LspService::build(Backend::new).finish();
        Server::new(read, write, socket).serve(service).await;
    } else {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::build(Backend::new).finish();
        Server::new(stdin, stdout, socket).serve(service).await;
    }
}
