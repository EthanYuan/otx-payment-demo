use crate::types::packed;

use ckb_jsonrpc_types::{JsonBytes, TransactionView, Uint32};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct OtxKeyPair {
    key_type: Uint32,
    key_data: Option<JsonBytes>,
    value_data: JsonBytes,
}

impl From<OtxKeyPair> for packed::OtxKeyPair {
    fn from(_json: OtxKeyPair) -> Self {
        todo!()
    }
}

impl From<packed::OtxKeyPair> for OtxKeyPair {
    fn from(_packed: packed::OtxKeyPair) -> OtxKeyPair {
        todo!()
    }
}

type OtxMap = Vec<OtxKeyPair>;
type OtxMapVec = Vec<OtxMap>;

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct OpenTransaction {
    meta: OtxMap,
    cell_deps: OtxMapVec,
    header_deps: OtxMapVec,
    inputs: OtxMapVec,
    witnesses: OtxMapVec,
    outputs: OtxMapVec,
}

impl From<OpenTransaction> for packed::OpenTransaction {
    fn from(_json: OpenTransaction) -> Self {
        todo!()
    }
}

impl From<packed::OpenTransaction> for OpenTransaction {
    fn from(_packed: packed::OpenTransaction) -> OpenTransaction {
        todo!()
    }
}

impl From<TransactionView> for OpenTransaction {
    fn from(tx_view: TransactionView) -> Self {
        todo!()
    }
}

impl From<OpenTransaction> for TransactionView {
    fn from(otx: OpenTransaction) -> Self {
        todo!()
    }
}
