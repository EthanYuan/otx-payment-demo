use crate::error::OtxError;
use crate::jsonrpc_types::constant::key_type::OTX_META_VERSION;
use crate::jsonrpc_types::{OpenTransaction, OtxKeyPair, OtxMap};

use anyhow::Result;
use ckb_jsonrpc_types::{JsonBytes, TransactionView, Uint32};
use ckb_types::prelude::{Entity, Pack};

pub fn tx_view_to_otx(
    tx_view: TransactionView,
    otx_meta_version: u32,
) -> Result<OpenTransaction, OtxError> {
    let open_tx_version: Uint32 = match otx_meta_version {
        0 => otx_meta_version.into(),
        _ => return Err(OtxError::VersionNotSupported(otx_meta_version.to_string())),
    };
    let key_type: Uint32 = OTX_META_VERSION.into();
    let meta = vec![OtxKeyPair::new(
        key_type,
        None,
        JsonBytes::from_bytes(open_tx_version.pack().as_bytes()),
    )];

    let cell_deps: Vec<OtxMap> = tx_view
        .inner
        .cell_deps
        .into_iter()
        .map(Into::into)
        .collect();

    let header_deps: Vec<OtxMap> = tx_view
        .inner
        .header_deps
        .into_iter()
        .map(Into::into)
        .collect();

    let inputs: Vec<OtxMap> = tx_view.inner.inputs.into_iter().map(Into::into).collect();

    let witnesses: Vec<OtxMap> = tx_view
        .inner
        .witnesses
        .into_iter()
        .map(Into::into)
        .collect();

    let outputs = tx_view
        .inner
        .outputs
        .into_iter()
        .zip(tx_view.inner.outputs_data.into_iter());
    let outputs: Vec<OtxMap> = outputs.map(Into::into).collect();

    Ok(OpenTransaction::new(
        meta.into(),
        cell_deps.into(),
        header_deps.into(),
        inputs.into(),
        witnesses.into(),
        outputs.into(),
    ))
}

pub fn otx_to_tx_view(_tx_view: OpenTransaction) -> Result<TransactionView> {
    todo!()
}
