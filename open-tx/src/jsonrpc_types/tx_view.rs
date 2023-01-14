use super::{HeaderDep, OutputData, Witness};
use crate::error::OtxError;
use crate::jsonrpc_types::constant::key_type::OTX_META_VERSION;
use crate::jsonrpc_types::{OpenTransaction, OtxKeyPair, OtxMap};

use anyhow::Result;
use ckb_jsonrpc_types::{CellDep, CellInput, CellOutput, JsonBytes, TransactionView, Uint32};
use ckb_types::constants::TX_VERSION;
use ckb_types::core::TransactionBuilder;
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

pub fn extract_ckb_tx(otx: OpenTransaction) -> Result<TransactionView, OtxError> {
    let witnesses = otx
        .witnesses
        .into_iter()
        .map(|witness| witness.try_into())
        .collect::<Result<Vec<Witness>, _>>()?;

    let inputs = otx
        .inputs
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<CellInput>, _>>()?;

    let outputs: Vec<(CellOutput, OutputData)> =
        otx.outputs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<(CellOutput, OutputData)>, _>>()?;
    let (outputs, outputs_data): (Vec<_>, Vec<_>) =
        outputs.into_iter().map(|(a, b)| (a, b)).unzip();

    let cell_deps = otx
        .cell_deps
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<CellDep>, _>>()?;

    let header_deps = otx
        .header_deps
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<HeaderDep>, _>>()?;

    let tx_view = TransactionBuilder::default()
        .version(TX_VERSION.pack())
        .witnesses(
            witnesses
                .into_iter()
                .map(|witness| witness.as_bytes().pack()),
        )
        .inputs(inputs.into_iter().map(Into::into))
        .outputs(outputs.into_iter().map(Into::into))
        .outputs_data(outputs_data.into_iter().map(Into::into))
        .cell_deps(cell_deps.into_iter().map(Into::into))
        .header_deps(header_deps.into_iter().map(|h| h.pack()))
        .build();
    Ok(tx_view.into())
}
