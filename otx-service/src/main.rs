use otx_pool::rpc::{OtxPoolRpcImpl, OtxPoolRpcServer};
use utils::const_definition::SERVICE_URI;

use jsonrpsee_http_server::HttpServerBuilder;

use std::net::SocketAddr;
use std::sync::mpsc::channel;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        // should recognize RUST_LOG_STYLE environment variable
        env_logger::Builder::from_default_env()
            .filter(None, log::LevelFilter::Info)
            .init();
    } else {
        env_logger::init();
    }

    start().await;
}

pub async fn start() {
    // bind address
    let bind: Vec<&str> = SERVICE_URI.split("//").collect();
    let bind_addr: SocketAddr = bind[1].parse().expect("listen address parsed");

    // start server
    let server = HttpServerBuilder::default()
        .max_response_body_size(u32::MAX)
        .build(vec![bind_addr].as_slice())
        .await
        .expect("build server");
    let server_handler = server
        .start(OtxPoolRpcImpl::new().into_rpc())
        .expect("Start jsonrpc http server");
    log::info!("OTX jsonrpc server started: {}", SERVICE_URI);

    // stop
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();
    log::info!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    server_handler
        .stop()
        .expect("stop server handle")
        .await
        .expect("join server handle");
    log::info!("Closing!");
}
