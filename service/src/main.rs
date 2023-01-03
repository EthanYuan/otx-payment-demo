use common_lib::const_definition::SERVICE_URI;
use otx_transaction_pool::rpc::OtxPoolRpc;
use otx_transaction_pool::rpc::OtxPoolRpcImpl;

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
    let rpc_impl = OtxPoolRpcImpl::new();

    let bind = "0.0.0.0:8118";
    let bind_addr: SocketAddr = bind.parse()?;
    let mut io_handler = IoHandler::new();
    io_handler.extend_with(rpc_impl.to_delegate());
    let server = ServerBuilder::new(io_handler)
        .cors(DomainsValidation::AllowOnly(vec![
            AccessControlAllowOrigin::Null,
            AccessControlAllowOrigin::Any,
        ]))
        .health_api(("/ping", "ping"))
        .start_http(&bind_addr)
        .expect("Start Jsonrpc HTTP service");
    println!("jsonrpc server started: {}", bind);
    log::info!("jsonrpc server started: {}", bind);

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();
    log::info!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    server.close();
    Ok(())
}
