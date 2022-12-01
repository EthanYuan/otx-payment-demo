use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::{EpochView, LocalNode, OutputsValidator, Transaction};
use ckb_types::H256;
use jsonrpc_core::types::{
    Call, Id, MethodCall, Output, Params, Request, Response, Value, Version,
};
use reqwest::blocking::Client;
use serde::{de::DeserializeOwned, Serialize};

pub struct CkbRpcClient {
    client: RpcClient,
}

impl CkbRpcClient {
    pub fn new(uri: String) -> Self {
        let client = RpcClient::new(uri);
        CkbRpcClient { client }
    }

    pub fn local_node_info(&self) -> Result<LocalNode> {
        request(&self.client, "local_node_info", ())
    }

    pub fn get_current_epoch(&self) -> Result<EpochView> {
        request(&self.client, "get_current_epoch", ())
    }

    pub fn generate_block(&self) -> Result<H256> {
        request(&self.client, "generate_block", ())
    }

    pub fn send_transaction(
        &self,
        tx: Transaction,
        outputs_validator: OutputsValidator,
    ) -> Result<H256> {
        request(&self.client, "send_transaction", (tx, outputs_validator))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RpcClient {
    client: Client,
    uri: String,
}

impl RpcClient {
    pub(crate) fn new(uri: String) -> Self {
        RpcClient {
            client: Client::new(),
            uri,
        }
    }

    pub(crate) fn rpc_exec(&self, request: &Request) -> Result<Response> {
        let http_response = self.client.post(self.uri.as_str()).json(request).send()?;

        if !http_response.status().is_success() {
            return Err(anyhow!("http response"));
        }

        http_response.json().map_err(anyhow::Error::new)
    }
}

fn request<T: Serialize, U: DeserializeOwned>(
    client: &RpcClient,
    method: &str,
    params: T,
) -> Result<U> {
    let request = build_request(method, params)?;
    let response = client.rpc_exec(&request)?;
    handle_response(response)
}

fn build_request<T: Serialize>(method: &str, params: T) -> Result<Request> {
    let request = Request::Single(Call::MethodCall(MethodCall {
        jsonrpc: Some(Version::V2),
        method: method.to_string(),
        params: parse_params(&params)?,
        id: Id::Num(42),
    }));
    Ok(request)
}

fn parse_params<T: Serialize>(params: &T) -> Result<Params> {
    let json = serde_json::to_value(params)?;

    match json {
        Value::Array(vec) => Ok(Params::Array(vec)),
        Value::Object(map) => Ok(Params::Map(map)),
        Value::Null => Ok(Params::None),
        _ => Err(anyhow!("parse params")),
    }
}

fn handle_response<T: DeserializeOwned>(response: Response) -> Result<T> {
    match response {
        Response::Single(output) => handle_output(output),
        _ => unreachable!(),
    }
}

fn handle_output<T: DeserializeOwned>(output: Output) -> Result<T> {
    let value = match output {
        Output::Success(succ) => succ.result,
        Output::Failure(_) => return Err(anyhow!("handle output: {:?}", output)),
    };

    serde_json::from_value(value).map_err(anyhow::Error::new)
}