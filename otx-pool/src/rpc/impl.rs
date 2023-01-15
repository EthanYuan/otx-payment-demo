use super::{OtxPoolRpcImpl, OtxPoolRpcServer};
use crate::error::InnerResult;

use ckb_types::prelude::Entity;
use otx_format::{jsonrpc_types::OpenTransaction, types::packed};

use async_trait::async_trait;
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::H256;
use jsonrpsee_core::RpcResult;

#[async_trait]
impl OtxPoolRpcServer for OtxPoolRpcImpl {
    async fn submit_otx(&self, otx: JsonBytes) -> RpcResult<String> {
        let otx = parse_otx(otx)?;
        Ok(format!("submit_otx: {:?}", otx))
    }

    async fn query_otx_by_id(&self, _id: H256) -> RpcResult<()> {
        Ok(())
    }
}

fn parse_otx(otx: JsonBytes) -> InnerResult<OpenTransaction> {
    let r = packed::OpenTransaction::from_slice(otx.as_bytes());
    r.map(Into::into).map_err(Into::into)
}
