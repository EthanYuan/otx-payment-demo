use super::{request, RpcClient};

use anyhow::Result;
use ckb_jsonrpc_types::JsonBytes;

pub struct ServiceRpcClient {
    client: RpcClient,
}

impl ServiceRpcClient {
    pub fn new(uri: String) -> Self {
        let client = RpcClient::new(uri);
        ServiceRpcClient { client }
    }

    pub fn submit_otx(&self, otx: JsonBytes) -> Result<String> {
        request(&self.client, "submit_otx", vec![otx])
    }
}
