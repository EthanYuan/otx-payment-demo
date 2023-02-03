use super::service::ServiceHandler;

use otx_plugin_protocol::{MessageFromHost, MessageFromPlugin, MessageType, PluginInfo};

use ckb_types::core::service::Request;
use crossbeam_channel::{bounded, select, unbounded, Sender};

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread::{self, JoinHandle};

pub type RequestHandler = Sender<Request<(u64, MessageFromHost), (u64, MessageFromPlugin)>>;
pub type NotifyHandler = Sender<MessageFromHost>;

#[derive(Clone, Debug)]
pub struct PluginState {
    pub binary_path: PathBuf,
    pub is_active: bool,
}

impl PluginState {
    pub fn new(binary_path: PathBuf, is_active: bool) -> PluginState {
        PluginState {
            binary_path,
            is_active,
        }
    }
}

pub struct PluginProcess {
    _plugin_process: Child,
    _stdin_thread: JoinHandle<()>,
    _stdout_thread: JoinHandle<()>,
}

pub struct PluginProxy {
    _state: PluginState,
    _info: PluginInfo,
    _process: PluginProcess,

    /// Send request to stdin thread, and expect a response from stdout thread.
    _request_handler: RequestHandler,

    /// Send notifaction to stdin thread.
    _nofify_handler: NotifyHandler,
}

impl PluginProxy {
    pub fn get_notify_handler(&self) -> NotifyHandler {
        self._nofify_handler.clone()
    }

    /// This function will create a temporary plugin process to fetch plugin information.
    pub fn get_plug_info(binary_path: PathBuf) -> Result<PluginInfo, String> {
        let mut child = Command::new(&binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| err.to_string())?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| String::from("Get stdin failed"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| String::from("Get stdout failed"))?;

        // request from host to plugin
        let request = (0u64, MessageFromHost::GetPluginInfo);
        let request_string = serde_json::to_string(&request).expect("Serialize request error");
        log::debug!("Send request to plugin: {}", request_string);
        stdin
            .write_all(format!("{}\n", request_string).as_bytes())
            .map_err(|err| err.to_string())?;
        stdin.flush().map_err(|err| err.to_string())?;

        // get response from plugin
        let mut buf_reader = BufReader::new(stdout);
        let mut response_string = String::new();
        buf_reader
            .read_line(&mut response_string)
            .map_err(|err| err.to_string())?;
        log::debug!("Receive response from plugin: {}", response_string.trim());
        let (id, response): (u64, MessageFromPlugin) =
            serde_json::from_str(&response_string).map_err(|err| err.to_string())?;

        if let (0u64, MessageFromPlugin::PluginInfo(plugin_info)) = (id, response) {
            Ok(plugin_info)
        } else {
            Err(format!(
                "Invalid response for get_info call to plugin {:?}, response: {}",
                binary_path, response_string
            ))
        }
    }

    pub fn start_process(
        plugin_state: PluginState,
        plugin_info: PluginInfo,
        service_handler: ServiceHandler,
    ) -> Result<PluginProxy, String> {
        let mut child = Command::new(plugin_state.binary_path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|err| err.to_string())?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| String::from("Get stdin failed"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| String::from("Get stdout failed"))?;

        // the host request channel receives request from host to plugin
        let (host_request_sender, host_request_receiver) = bounded(1);

        // the plugin response channel receives response from plugin,
        // it cooperates with the host request channel to complete the request-response pair
        let (plugin_response_sender, plugin_response_receiver) = bounded(1);

        // the channel sends responses from the host to plugin
        let (host_response_sender, host_response_receiver) = bounded(1);

        // the channel sends notifications from the host to plugin
        let (host_notify_sender, host_notify_receiver) = unbounded();

        let plugin_name = plugin_info.name.clone();
        // this thread processes stdin information from host to plugin
        let stdin_thread = thread::spawn(move || {
            let handle_host_response_msg =
                |stdin: &mut ChildStdin, (id, response)| -> Result<bool, String> {
                    let response_string =
                        serde_json::to_string(&(id, response)).expect("Serialize response error");
                    log::debug!("Send response to plugin: {}", response_string);
                    stdin
                        .write_all(format!("{}\n", response_string).as_bytes())
                        .map_err(|err| err.to_string())?;
                    stdin.flush().map_err(|err| err.to_string())?;
                    Ok(false)
                };

            let handle_host_notify_msg = |stdin: &mut ChildStdin, msg| -> Result<bool, String> {
                let notify_string = serde_json::to_string(&msg).expect("Serialize response error");
                log::debug!("Send response to plugin: {}", notify_string);
                stdin
                    .write_all(format!("{}\n", notify_string).as_bytes())
                    .map_err(|err| err.to_string())?;
                stdin.flush().map_err(|err| err.to_string())?;
                Ok(false)
            };

            let mut do_select = || -> Result<bool, String> {
                select! {
                    // request from host to plugin
                    recv(host_request_receiver) -> msg => {
                        match msg {
                            Ok(Request { responder, arguments }) => {
                                let request_string = serde_json::to_string(&arguments).expect("Serialize request error");
                                log::debug!("Send request to plugin: {}", request_string);
                                stdin.write_all(format!("{}\n", request_string).as_bytes()).map_err(|err| err.to_string())?;
                                stdin.flush().map_err(|err| err.to_string())?;
                                loop {
                                    select!{
                                        recv(plugin_response_receiver) -> msg => {
                                            match msg {
                                                Ok(response) => {
                                                    responder.send(response).map_err(|err| err.to_string())?;
                                                    return Ok(false);
                                                }
                                                Err(err) => {
                                                    return Err(err.to_string());
                                                }
                                            }
                                        },
                                        recv(host_response_receiver) -> msg => {
                                            match msg {
                                                Ok(msg) => {
                                                    handle_host_response_msg(&mut stdin, msg)?;
                                                },
                                                Err(err) => {
                                                    return Err(err.to_string());
                                                }
                                            }
                                        },
                                        recv(host_notify_receiver) -> msg => {
                                            match msg {
                                                Ok(msg) => {
                                                    handle_host_notify_msg(&mut stdin, msg)?;
                                                },
                                                Err(err) => {
                                                    return Err(err.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(err) => Err(err.to_string())
                        }
                    }
                    // repsonse from plugin to host (ServiceProvider)
                    recv(host_response_receiver) -> msg_result => {
                        match msg_result {
                            Ok(msg) => handle_host_response_msg(&mut stdin, msg),
                            Err(err) => Err(err.to_string())
                        }
                    }
                    // notify from plugin to host
                    recv(host_notify_receiver) -> msg_result => {
                        match msg_result {
                            Ok(msg) => handle_host_notify_msg(&mut stdin, msg),
                            Err(err) => Err(err.to_string())
                        }
                    }
                }
            };
            loop {
                match do_select() {
                    Ok(true) => {
                        break;
                    }
                    Ok(false) => (),
                    Err(err) => {
                        log::info!("plugin {} stdin error: {}", plugin_name, err);
                        break;
                    }
                }
            }
        });

        let plugin_name = plugin_info.name.clone();
        let mut buf_reader = BufReader::new(stdout);
        let stdout_thread = thread::spawn(move || {
            let mut do_recv = || -> Result<bool, String> {
                let mut content = String::new();
                if buf_reader
                    .read_line(&mut content)
                    .map_err(|err| err.to_string())?
                    == 0
                {
                    // EOF
                    return Ok(true);
                }

                let (id, message_from_plugin): (u64, MessageFromPlugin) =
                    serde_json::from_str(&content).map_err(|err| err.to_string())?;
                match message_from_plugin.get_message_type() {
                    MessageType::Response => {
                        // Receive response from plugin
                        log::debug!("Receive response from plugin: {}", content.trim());
                        plugin_response_sender
                            .send((id, message_from_plugin))
                            .map_err(|err| err.to_string())?;
                    }
                    MessageType::Request => {
                        // Handle request from plugin
                        log::debug!("Receive request from plugin: {}", content.trim());
                        log::debug!("Sending request to ServiceProvider");
                        let message_from_host =
                            Request::call(&service_handler, message_from_plugin).ok_or_else(
                                || String::from("Send request to ServiceProvider failed"),
                            )?;
                        log::debug!("Received response from ServiceProvider");
                        host_response_sender
                            .send((id, message_from_host))
                            .map_err(|err| err.to_string())?;
                    }
                    MessageType::Notify => {
                        unreachable!()
                    }
                }

                Ok(false)
            };
            loop {
                match do_recv() {
                    Ok(true) => {
                        log::info!("plugin {} quit", plugin_name);
                        break;
                    }
                    Ok(false) => {}
                    Err(err) => {
                        log::warn!("plugin {} stdout error: {}", plugin_name, err);
                        break;
                    }
                }
            }
        });

        let process = PluginProcess {
            _plugin_process: child,
            _stdin_thread: stdin_thread,
            _stdout_thread: stdout_thread,
        };

        Ok(PluginProxy {
            _state: plugin_state,
            _info: plugin_info,
            _process: process,
            _request_handler: host_request_sender,
            _nofify_handler: host_notify_sender,
        })
    }
}

impl Drop for PluginProxy {
    fn drop(&mut self) {
        // TODO: send term signal to the process
    }
}
