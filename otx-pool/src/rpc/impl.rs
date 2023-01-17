use super::{OtxPoolRpcImpl, OtxPoolRpcServer};
use crate::pool::Id;

use otx_format::jsonrpc_types::OpenTransaction;

use async_trait::async_trait;
use ckb_jsonrpc_types::JsonBytes;
use jsonrpsee_core::RpcResult;

#[async_trait]
impl OtxPoolRpcServer for OtxPoolRpcImpl {
    async fn submit_otx(&self, otx: JsonBytes) -> RpcResult<Id> {
        self.otx_pool.insert(otx).map_err(Into::into)
    }

    async fn query_otx_by_id(&self, id: Id) -> RpcResult<Option<OpenTransaction>> {
        Ok(self.otx_pool.get_otx_by_id(id))
    }
}
