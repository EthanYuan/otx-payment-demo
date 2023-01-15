mod r#impl;

use super::otx_pool::{Id, OtxPool};

use otx_format::jsonrpc_types::OpenTransaction;

use ckb_jsonrpc_types::JsonBytes;
use jsonrpsee_core::RpcResult;
use jsonrpsee_proc_macros::rpc;

#[rpc(server)]
pub trait OtxPoolRpc {
    #[method(name = "submit_otx")]
    async fn submit_otx(&self, otx: JsonBytes) -> RpcResult<Id>;

    #[method(name = "query_otx_by_id")]
    async fn query_otx_by_id(&self, id: Id) -> RpcResult<Option<OpenTransaction>>;
}

pub struct OtxPoolRpcImpl {
    otx_pool: OtxPool,
}

impl OtxPoolRpcImpl {
    pub fn new() -> Self {
        OtxPoolRpcImpl {
            otx_pool: OtxPool::new(),
        }
    }
}

impl Default for OtxPoolRpcImpl {
    fn default() -> Self {
        Self::new()
    }
}
