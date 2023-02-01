use super::service::ServiceHandler;

use otx_plugin_protocol::{MessageFromHost, MessageFromPlugin, PluginInfo};

use ckb_types::core::service::Request;
use crossbeam_channel::{bounded, select, Sender};

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};

pub type RequestHandler = Sender<Request<(u64, MessageFromHost), (u64, MessageFromPlugin)>>;

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

    // Send message to stdin thread, and expect a response from stdout thread
    _plugin_handler: RequestHandler,
}

pub struct PluginProxy {
    _state: PluginState,
    _info: PluginInfo,
    _process: PluginProcess,
}

impl PluginProxy {
    pub fn get_plug_info(binary_path: PathBuf) -> Result<PluginInfo, String> {
        // create temp plugin process
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
        _service_handler: ServiceHandler,
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

        let (request_sender, request_receiver) = bounded(1);
        let (_stdout_sender, stdout_receiver) = bounded(1);
        // let (service_sender, service_receiver) = bounded(1);
        // let (stop_sender, stop_receiver) = bounded(1);

        let plugin_name = plugin_info.name.clone();
        let plugin_name_2 = plugin_info.name.clone();

        // This thread processes stdin information from host to plugin
        let stdin_thread = thread::spawn(move || {
            // let handle_service_channel_msgs =
            //     |stdin: &mut ChildStdin, (id, response)| -> Result<bool, String> {
            //         let response = (id, response);
            //         let response_string =
            //             serde_json::to_string(&response).expect("Serialize response error");
            //         log::debug!("Send response to plugin: {}", response_string);
            //         stdin
            //             .write_all(format!("{}\n", response_string).as_bytes())
            //             .map_err(|err| err.to_string())?;
            //         stdin.flush().map_err(|err| err.to_string())?;
            //         Ok(false)
            //     };

            let mut do_select = || -> Result<bool, String> {
                select! {
                    // request from host to plugin
                    recv(request_receiver) -> msg_result => {
                        match msg_result {
                            Ok(Request { responder, arguments }) => {
                                let request_string = serde_json::to_string(&arguments).expect("Serialize request error");
                                log::debug!("Send request to plugin: {}", request_string);
                                stdin.write_all(format!("{}\n", request_string).as_bytes()).map_err(|err| err.to_string())?;
                                stdin.flush().map_err(|err| err.to_string())?;
                                loop {
                                    select!{
                                        recv(stdout_receiver) -> msg_result => {
                                            match msg_result {
                                                Ok(response) => {
                                                    responder.send(response).map_err(|err| err.to_string())?;
                                                    return Ok(false);
                                                }
                                                Err(err) => {
                                                    return Err(err.to_string());
                                                }
                                            }
                                        },
                                        // recv(service_receiver) -> msg_result => {
                                        //     match msg_result {
                                        //         Ok(msg) => {
                                        //             handle_service_channel_msgs(&mut stdin, msg)?;
                                        //         },
                                        //         Err(err) => {
                                        //             return Err(err.to_string());
                                        //         }
                                        //     }
                                        // }
                                    }
                                }
                            }
                            Err(err) => Err(err.to_string())
                        }
                    }
                    // repsonse from plugin to host (ServiceProvider)
                    // recv(service_receiver) -> msg_result => {
                    //     match msg_result {
                    //         Ok(msg) => handle_service_channel_msgs(&mut stdin, msg),
                    //         Err(err) => Err(err.to_string())
                    //     }
                    // }
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

        let mut buf_reader = BufReader::new(stdout);
        let stdout_thread = thread::spawn(move || {
            let mut do_recv = || -> Result<bool, String> {
                // if stop_receiver.try_recv().is_ok() {
                //     return Ok(true);
                // }
                let mut content = String::new();
                if buf_reader
                    .read_line(&mut content)
                    .map_err(|err| err.to_string())?
                    == 0
                {
                    // EOF
                    return Ok(true);
                }
                // let result: Result<JsonrpcResponse, _> = serde_json::from_str(&content);
                // if let Ok(jsonrpc_response) = result {
                //     // Receive response from plugin
                //     log::debug!("Receive response from plugin: {}", content.trim());
                //     let (id, response) = jsonrpc_response.try_into()?;
                //     stdout_sender
                //         .send((id, response))
                //         .map_err(|err| err.to_string())?;
                // } else {
                //     // Handle request from plugin
                //     log::debug!("Receive request from plugin: {}", content.trim());
                //     let jsonrpc_request: JsonrpcRequest =
                //         serde_json::from_str(&content).map_err(|err| err.to_string())?;
                //     let (id, request) = jsonrpc_request.try_into()?;
                //     let service_request = ServiceRequest::Request {
                //         is_from_plugin: true,
                //         plugin_name: config.name.clone(),
                //         request,
                //     };
                //     log::debug!("Sending request to ServiceProvider");
                //     if let ServiceResponse::Response(response) =
                //         Request::call(&service_handler, service_request)
                //             .ok_or_else(|| String::from("Send request to ServiceProvider failed"))?
                //     {
                //         log::debug!("Received response from ServiceProvider");
                //         service_sender
                //             .send((id, response))
                //             .map_err(|err| err.to_string())?;
                //     }
                // }
                Ok(false)
            };
            loop {
                match do_recv() {
                    Ok(true) => {
                        log::info!("plugin {} quit", plugin_name_2);
                        break;
                    }
                    Ok(false) => {}
                    Err(err) => {
                        log::warn!("plugin {} stdout error: {}", plugin_name_2, err);
                        break;
                    }
                }
            }
        });

        let process = PluginProcess {
            _plugin_process: child,
            _stdin_thread: stdin_thread,
            _stdout_thread: stdout_thread,
            _plugin_handler: request_sender,
        };

        Ok(PluginProxy {
            _state: plugin_state,
            _info: plugin_info,
            _process: process,
        })
    }
}

impl Drop for PluginProxy {
    fn drop(&mut self) {
        // TODO: send term signal to the process
    }
}
