use otx_pool::rpc::{OtxPoolRpc, OtxPoolRpcImpl};
use utils::const_definition::SERVICE_URI;

use anyhow::Result;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_server_utils::cors::AccessControlAllowOrigin;
use jsonrpc_server_utils::hosts::DomainsValidation;

use std::net::SocketAddr;
use std::sync::mpsc::channel;

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        // should recognize RUST_LOG_STYLE environment variable
        env_logger::Builder::from_default_env()
            .filter(None, log::LevelFilter::Info)
            .init();
    } else {
        env_logger::init();
    }

    start()
}

pub fn start() -> Result<()> {
    // bind
    let bind: Vec<&str> = SERVICE_URI.split("//").collect();
    let bind_addr: SocketAddr = bind[1].parse()?;

    // handler
    let rpc_impl = OtxPoolRpcImpl::new();
    let mut io_handler = IoHandler::new();
    io_handler.extend_with(rpc_impl.to_delegate());

    // init server
    let server = ServerBuilder::new(io_handler)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
            AccessControlAllowOrigin::Any,
        ]))
        .health_api(("/ping", "ping"))
        .start_http(&bind_addr)
        .expect("Start Jsonrpc HTTP service");
    log::info!("jsonrpc server started: {}", SERVICE_URI);

    // close
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();
    log::info!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    server.close();
    Ok(())
}
