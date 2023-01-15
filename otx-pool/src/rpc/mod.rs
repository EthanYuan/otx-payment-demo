mod r#impl;

use super::otx_pool::OtxPool;

use ckb_jsonrpc_types::JsonBytes;
use ckb_types::H256;
use jsonrpsee_core::RpcResult;
use jsonrpsee_proc_macros::rpc;

#[rpc(server)]
pub trait OtxPoolRpc {
    #[method(name = "submit_otx")]
    async fn submit_otx(&self, otx: JsonBytes) -> RpcResult<String>;

    #[method(name = "query_otx_by_id")]
    async fn query_otx_by_id(&self, id: H256) -> RpcResult<()>;
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
