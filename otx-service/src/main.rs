use otx_pool::{
    notify::NotifyService,
    plugin::manager::PluginManager,
    rpc::{OtxPoolRpcImpl, OtxPoolRpcServer},
};
use utils::const_definition::SERVICE_URI;

use ckb_async_runtime::new_global_runtime;
use jsonrpsee_http_server::HttpServerBuilder;
pub use tokio;
pub use tokio::runtime::Runtime;

use std::time::Duration;
use std::{net::SocketAddr, path::Path};

pub const MESSAGE_CHANNEL_SIZE: usize = 1024;
const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);
pub const PLUGINS_DIRNAME: &str = "plugins";

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
    // runtime handle
    let (handle, runtime) = new_global_runtime();

    // bind address
    let bind: Vec<&str> = SERVICE_URI.split("//").collect();
    let bind_addr: SocketAddr = bind[1].parse().expect("listen address parsed");

    // start notify service
    let notify_service = NotifyService::new();
    let notify_ctrl = notify_service.start(handle);

    // interval loop
    // let interval_child = thread::spawn(move || {
    //     let mut interval = time::interval(Duration::from_millis(10));

    //     interval.tick().await; // ticks immediately
    //     interval.tick().await; // ticks after 10ms
    //     interval.tick().await; // ticks after 10ms
    //     thread_tx.send(id).unwrap();

    //     // Sending is a non-blocking operation, the thread will continue
    //     // immediately after sending its message
    //     println!("thread {} finished", id);
    // });

    // init otx pool rpc
    let otx_pool_rpc = OtxPoolRpcImpl::new(notify_ctrl.clone());

    // start rpc server
    let server = HttpServerBuilder::default()
        .max_response_body_size(u32::MAX)
        .build(vec![bind_addr].as_slice())
        .await
        .expect("build server");
    let server_handler = server
        .start(otx_pool_rpc.into_rpc())
        .expect("Start jsonrpc http server");
    log::info!("OTX jsonrpc server started: {}", SERVICE_URI);

    // init plugins
    let plugin_manager = PluginManager::init(Path::new("./")).unwrap();
    let plugins = plugin_manager.plugin_configs();
    println!("{:?}", plugins.get("plugin demo"));
    log::info!("plugins count: {:?}", plugins.len());

    // test
    let mut rx = notify_ctrl.subscribe_new_open_tx("main-test").await;
    let otx = rx.recv().await;
    println!("{:?}", otx);

    // stop rpc server
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();
    log::info!("Waiting for Ctrl-C...");
    rx.recv().expect("Could not receive from channel.");
    server_handler
        .stop()
        .expect("stop server handle")
        .await
        .expect("join server handle");
    log::info!("Closing!");

    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
}
