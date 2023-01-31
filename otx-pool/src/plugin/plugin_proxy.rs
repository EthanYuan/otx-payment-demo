use otx_plugin_protocol::{MessageFromHost, MessageFromPlugin, PluginInfo};

use ckb_types::core::service::Request;
use crossbeam_channel::Sender;

use std::path::PathBuf;
use std::process::Child;
use std::thread::JoinHandle;

pub type RequestHandler = Sender<Request<(u64, MessageFromHost), (u64, MessageFromPlugin)>>;

pub struct PluginProxy {
    _binary_path: PathBuf,
    _is_active: bool,
    _info: PluginInfo,
    _plugin_process: Child,
    _stdin_thread: JoinHandle<()>,
    _stdout_thread: JoinHandle<()>,

    // Send message to stdin thread, and expect a response from stdout thread
    _plugin_handler: RequestHandler,
}
