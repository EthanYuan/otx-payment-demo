use otx_pool::{
    notify::NotifyService,
    plugin::manager::PluginManager,
    rpc::{OtxPoolRpcImpl, OtxPoolRpcServer},
};
use utils::const_definition::SERVICE_URI;

use ckb_async_runtime::new_global_runtime;
use jsonrpsee_http_server::HttpServerBuilder;
use tokio::time::{self, Duration};

use std::{net::SocketAddr, path::Path};

pub const MESSAGE_CHANNEL_SIZE: usize = 1024;
const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
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
    let notify_ctrl = notify_service.start(handle.clone());

    // interval loop
    let notifier = notify_ctrl.clone();
    let interval_handler = handle.spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            notifier.notify_interval();
        }
    });

    // init plugins
    let plugin_manager = PluginManager::init(notify_ctrl.clone(), Path::new("./")).unwrap();
    let plugins = plugin_manager.plugin_configs();
    log::info!("actived plugins count: {:?}", plugins.len());

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

    // test
    // let mut rx = notify_ctrl.subscribe_new_open_tx("main-test").await;
    // let otx = rx.recv().await;
    // println!("{:?}", otx);

    // stop
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).unwrap()).unwrap();
    log::info!("Waiting for Ctrl-C...");
    rx.recv().expect("Receive Ctrl-C from channel.");

    interval_handler.abort();
    server_handler
        .stop()
        .expect("stop server handle")
        .await
        .expect("join server handle");
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);

    log::info!("Closing!");
}
