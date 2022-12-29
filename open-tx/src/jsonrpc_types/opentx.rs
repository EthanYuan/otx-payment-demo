use crate::types::packed::{self, OpenTransactionBuilder, OtxMapBuilder, OtxMapVecBuilder};

use ckb_jsonrpc_types::{JsonBytes, TransactionView, Uint32};
use ckb_types::prelude::{Builder, Pack, Unpack};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct OtxKeyPair {
    key_type: Uint32,
    key_data: Option<JsonBytes>,
    value_data: JsonBytes,
}

impl From<OtxKeyPair> for packed::OtxKeyPair {
    fn from(json: OtxKeyPair) -> Self {
        packed::OtxKeyPairBuilder::default()
            .key_type(json.key_type.pack())
            .key_data(json.key_data.map(|data| data.into_bytes()).pack())
            .value_data(json.value_data.into_bytes().pack())
            .build()
    }
}

impl From<packed::OtxKeyPair> for OtxKeyPair {
    fn from(packed: packed::OtxKeyPair) -> Self {
        OtxKeyPair {
            key_type: packed.key_type().unpack(),
            key_data: packed.key_data().to_opt().map(Into::into),
            value_data: packed.value_data().into(),
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
struct OtxMap(Vec<OtxKeyPair>);

impl From<OtxMap> for packed::OtxMap {
    fn from(json: OtxMap) -> Self {
        let map: Vec<packed::OtxKeyPair> = json.0.into_iter().map(Into::into).collect();
        OtxMapBuilder::default().set(map).build()
    }
}

impl From<packed::OtxMap> for OtxMap {
    fn from(packed: packed::OtxMap) -> Self {
        OtxMap(packed.into_iter().map(Into::into).collect())
    }
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
struct OtxMapVec(Vec<OtxMap>);

impl From<OtxMapVec> for packed::OtxMapVec {
    fn from(json: OtxMapVec) -> Self {
        let map_vec: Vec<packed::OtxMap> = json.0.into_iter().map(Into::into).collect();
        OtxMapVecBuilder::default().set(map_vec).build()
    }
}

impl From<packed::OtxMapVec> for OtxMapVec {
    fn from(packed: packed::OtxMapVec) -> Self {
        OtxMapVec(packed.into_iter().map(Into::into).collect())
    }
}

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
    fn from(json: OpenTransaction) -> Self {
        OpenTransactionBuilder::default()
            .meta(json.meta.into())
            .cell_deps(json.cell_deps.into())
            .header_deps(json.header_deps.into())
            .inputs(json.inputs.into())
            .witnesses(json.witnesses.into())
            .outputs(json.outputs.into())
            .build()
    }
}

impl From<packed::OpenTransaction> for OpenTransaction {
    fn from(packed: packed::OpenTransaction) -> Self {
        OpenTransaction {
            meta: packed.meta().into(),
            cell_deps: packed.cell_deps().into(),
            header_deps: packed.header_deps().into(),
            inputs: packed.inputs().into(),
            witnesses: packed.witnesses().into(),
            outputs: packed.outputs().into(),
        }
    }
}

impl From<OpenTransaction> for TransactionView {
    fn from(otx: OpenTransaction) -> Self {
        todo!()
    }
}

impl From<TransactionView> for OpenTransaction {
    fn from(tx_view: TransactionView) -> Self {
        todo!()
    }
}

