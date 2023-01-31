use super::plugin_proxy::PluginProxy;

use ckb_types::core::service::Request;
use crossbeam_channel::Sender;
use otx_plugin_protocol::{MessageFromHost, MessageFromPlugin};
use std::thread::JoinHandle;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub type ServiceHandler = Sender<Request<MessageFromPlugin, MessageFromHost>>;

pub struct PluginManager {
    _plugin_dir: PathBuf,
    _plugins: HashMap<String, PluginProxy>,
    _service_provider: ServiceProvider,
    _jsonrpc_id: Arc<AtomicU64>,
}

struct ServiceProvider {
    _handler: ServiceHandler,
    _thread: JoinHandle<()>,
}
