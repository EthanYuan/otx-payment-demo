use super::constant::key_type::{
    OTX_CELL_DEP_TYPE, OTX_HEADER_DEP_HASH, OTX_INPUT_SINCE, OTX_OUTPOINT, OTX_OUTPUT_CAPACITY,
    OTX_OUTPUT_DATA, OTX_OUTPUT_LOCK_ARGS, OTX_OUTPUT_LOCK_CODE_HASH, OTX_OUTPUT_LOCK_HASH_TYPE,
    OTX_OUTPUT_TYPE_ARGS, OTX_OUTPUT_TYPE_CODE_HASH, OTX_OUTPUT_TYPE_HASH_TYPE,
    OTX_WITNESS_RAW,
};
use crate::types::packed::{self, OpenTransactionBuilder, OtxMapBuilder, OtxMapVecBuilder};

use ckb_jsonrpc_types::{CellDep, CellInput, CellOutput, JsonBytes, Uint32};
use ckb_types::core::{DepType, ScriptHashType};
use ckb_types::{self, prelude::*, H256};
use serde::{Deserialize, Serialize};

pub type HeaderDep = H256;
pub type Witness = JsonBytes;
pub type OutputData = JsonBytes;

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct OtxKeyPair {
    key_type: Uint32,
    key_data: Option<JsonBytes>,
    value_data: JsonBytes,
}

impl OtxKeyPair {
    pub fn new(key_type: Uint32, key_data: Option<JsonBytes>, value_data: JsonBytes) -> Self {
        OtxKeyPair {
            key_type,
            key_data,
            value_data,
        }
    }
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
pub struct OtxMapVec(Vec<OtxMap>);

impl IntoIterator for OtxMapVec {
    type Item = OtxMap;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<OtxMap>> for OtxMapVec {
    fn from(vec: Vec<OtxMap>) -> Self {
        OtxMapVec(vec)
    }
}

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

impl OpenTransaction {
    pub fn new(
        meta: OtxMap,
        cell_deps: OtxMapVec,
        header_deps: OtxMapVec,
        inputs: OtxMapVec,
        witnesses: OtxMapVec,
        outputs: OtxMapVec,
    ) -> Self {
        OpenTransaction {
            meta,
            cell_deps,
            header_deps,
            inputs,
            witnesses,
            outputs,
        }
    }
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

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct OtxMap(Vec<OtxKeyPair>);

impl IntoIterator for OtxMap {
    type Item = OtxKeyPair;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<OtxKeyPair>> for OtxMap {
    fn from(vec: Vec<OtxKeyPair>) -> Self {
        OtxMap(vec)
    }
}

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

impl From<CellDep> for OtxMap {
    fn from(cell_dep: CellDep) -> Self {
        let out_point: ckb_types::packed::OutPoint = cell_dep.out_point.into();
        let out_point = OtxKeyPair::new(
            OTX_OUTPOINT.into(),
            None,
            JsonBytes::from_bytes(out_point.as_bytes()),
        );
        let dep_type: DepType = cell_dep.dep_type.into();
        let dep_type: packed::Byte = dep_type.into();
        let dep_type = OtxKeyPair::new(
            OTX_CELL_DEP_TYPE.into(),
            None,
            JsonBytes::from_bytes(dep_type.as_bytes()),
        );
        vec![out_point, dep_type].into()
    }
}

impl From<HeaderDep> for OtxMap {
    fn from(header_dep: HeaderDep) -> Self {
        let header_dep = OtxKeyPair::new(
            OTX_HEADER_DEP_HASH.into(),
            None,
            JsonBytes::from_bytes(header_dep.pack().as_bytes()),
        );
        vec![header_dep].into()
    }
}

impl From<Witness> for OtxMap {
    fn from(witness: Witness) -> Self {
        let witness = OtxKeyPair::new(OTX_WITNESS_RAW.into(), None, witness);
        vec![witness].into()
    }
}

impl From<CellInput> for OtxMap {
    fn from(cell_input: CellInput) -> Self {
        let previous_output: ckb_types::packed::OutPoint = cell_input.previous_output.into();
        let previous_output = OtxKeyPair::new(
            OTX_OUTPOINT.into(),
            None,
            JsonBytes::from_bytes(previous_output.as_bytes()),
        );
        let since = cell_input.since.pack();
        let since = OtxKeyPair::new(
            OTX_INPUT_SINCE.into(),
            None,
            JsonBytes::from_bytes(since.as_bytes()),
        );
        vec![previous_output, since].into()
    }
}

impl From<(CellOutput, OutputData)> for OtxMap {
    fn from(output: (CellOutput, OutputData)) -> Self {
        let capacity = OtxKeyPair::new(
            OTX_OUTPUT_CAPACITY.into(),
            None,
            JsonBytes::from_bytes(output.0.capacity.pack().as_bytes()),
        );
        let lock_code_hash = OtxKeyPair::new(
            OTX_OUTPUT_LOCK_CODE_HASH.into(),
            None,
            JsonBytes::from_bytes(output.0.lock.code_hash.pack().as_bytes()),
        );
        let lock_hash_type: ScriptHashType = output.0.lock.hash_type.into();
        let lock_hash_type: packed::Byte = lock_hash_type.into();
        let lock_hash_type = OtxKeyPair::new(
            OTX_OUTPUT_LOCK_HASH_TYPE.into(),
            None,
            JsonBytes::from_bytes(lock_hash_type.as_bytes()),
        );
        let lock_args = OtxKeyPair::new(OTX_OUTPUT_LOCK_ARGS.into(), None, output.0.lock.args);
        let mut map = vec![capacity, lock_code_hash, lock_hash_type, lock_args];

        if let Some(type_) = output.0.type_ {
            let type_code_hash = OtxKeyPair::new(
                OTX_OUTPUT_TYPE_CODE_HASH.into(),
                None,
                JsonBytes::from_bytes(type_.code_hash.pack().as_bytes()),
            );
            map.push(type_code_hash);
            let type_hash_type: ScriptHashType = type_.hash_type.into();
            let type_hash_type: packed::Byte = type_hash_type.into();
            let type_hash_type = OtxKeyPair::new(
                OTX_OUTPUT_TYPE_HASH_TYPE.into(),
                None,
                JsonBytes::from_bytes(type_hash_type.as_bytes()),
            );
            map.push(type_hash_type);
            let type_args = OtxKeyPair::new(OTX_OUTPUT_TYPE_ARGS.into(), None, type_.args);
            map.push(type_args);
        };

        let data = OtxKeyPair::new(OTX_OUTPUT_DATA.into(), None, output.1);
        map.push(data);

        map.into()
    }
}
