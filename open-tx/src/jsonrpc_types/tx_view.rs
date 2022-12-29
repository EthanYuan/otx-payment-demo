use crate::jsonrpc_types::OpenTransaction;

use anyhow::Result;
use ckb_jsonrpc_types::TransactionView;

pub fn tx_view_to_otx(
    _tx_view: TransactionView,
    _otx_meta_version: u32,
) -> Result<OpenTransaction> {
    // let open_tx_version:Uint32 = open_tx_version.into();
    // let meta = vec![OtxKeyPair::new(
    //     key_type::OTX_META_VERSION,
    //     None,
    //     open_tx_version
    // )
    // ];
    // let cell_deps = tx_view
    //     .inner
    //     .cell_deps
    //     .into_iter()
    //     .map(|cell_dep| {})
    //     .collect();

    // OpenTransaction {
    //     meta: (),
    //     cell_deps: (),
    //     header_deps: (),
    //     inputs: (),
    //     witnesses: (),
    //     outputs: (),
    // }
    todo!()
}

pub fn otx_to_tx_view(_tx_view: OpenTransaction) -> Result<TransactionView> {
    todo!()
}
