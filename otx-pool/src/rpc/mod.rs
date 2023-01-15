use super::otx_pool::OtxPool;

use ckb_jsonrpc_types::JsonBytes;
use ckb_types::H256;
use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;

#[rpc(server)]
pub trait OtxPoolRpc {
    #[rpc(name = "submit_otx")]
    fn submit_otx(&self, otx: JsonBytes) -> RpcResult<String>;

    #[rpc(name = "query_otx_by_id")]
    fn query_otx_by_id(&self, id: H256) -> RpcResult<()>;
}

pub struct OtxPoolRpcImpl {
    _otx_pool: OtxPool,
}

impl OtxPoolRpcImpl {
    pub fn new() -> Self {
        OtxPoolRpcImpl {
            _otx_pool: OtxPool::new(),
        }
    }
}

impl Default for OtxPoolRpcImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl OtxPoolRpc for OtxPoolRpcImpl {
    fn submit_otx(&self, _otx: JsonBytes) -> RpcResult<String> {
        Ok("submit_otx".to_string())
    }
    fn query_otx_by_id(&self, _id: H256) -> RpcResult<()> {
        Ok(())
    }
}
