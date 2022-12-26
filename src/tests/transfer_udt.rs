use super::super::IntegrationTest;

use common_lib::ckb_cli::ckb_cli_transfer_ckb;
use common_lib::instruction::dump_data;
use common_lib::instruction::mercury::{issue_udt_1, prepare_udt};
use common_lib::lock::omni::TxInfo;
use common_lib::mercury_client_rpc::MercuryRpcClient;
use service::{AddInputArgs, AddOutputArgs, OtxService};
use wallet::Wallet;

use common_lib::const_definition::{
    CKB_URI, MERCURY_URI, UDT_1_HASH, UDT_1_HOLDER_ACP_ADDRESS, UDT_1_HOLDER_PK,
    UDT_1_HOLDER_SECP_ADDRESS, XUDT_DEVNET_TYPE_HASH,
};

use anyhow::Result;
use ckb_types::{
    bytes::Bytes,
    core::{capacity_bytes, Capacity, ScriptHashType},
    packed::{Byte32, CellOutput, OutPoint, Script},
    prelude::*,
};
use core_rpc_types::{GetBalancePayload, JsonItem};

use std::collections::HashSet;

inventory::submit!(IntegrationTest {
    name: "test_issue_udt",
    test_fn: test_issue_udt
});
fn test_issue_udt() {
    // prepare udt
    issue_udt_1().unwrap();
    let _udt_hash = UDT_1_HASH.get().unwrap();
    let acp_address_with_udt = UDT_1_HOLDER_ACP_ADDRESS.get().unwrap();
    let _acp_address_pk = UDT_1_HOLDER_PK.get().unwrap();

    let payload = GetBalancePayload {
        item: JsonItem::Address(acp_address_with_udt.to_string()),
        asset_infos: HashSet::new(),
        extra: None,
        tip_block_number: None,
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    let response = mercury_client.get_balance(payload).unwrap();
    assert_eq!(response.balances.len(), 2);
    assert_eq!(0u128, response.balances[0].free.into());
    assert_eq!(142_0000_0000u128, response.balances[0].occupied.into());
    assert_eq!(0u128, response.balances[0].frozen.into());
    assert_eq!(200_0000_0000u128, response.balances[1].free.into());
}

inventory::submit!(IntegrationTest {
    name: "test_burn_udt",
    test_fn: test_burn_udt
});
fn test_burn_udt() {
    // {
    //     inputs: [
    //         {capacity: 144, data: 51, type: xudt z, lock: Bob},
    //         {capacity: 100, data: "", type: "", lock: Z}
    //     ],
    //     outputs: [
    //         {capacity: 144, data: 51-51, type: xudt z, lock: Bob},
    //         {capacity: 99, data: "", type: "", lock: Z} ]
    // }

    let open_tx = bob_build_signed_otx().unwrap();

    let z_service = OtxService::new(vec![], CKB_URI);
    let tx_hash = ckb_cli_transfer_ckb(z_service.signer.get_secp_address(), 100).unwrap();

    // builder in Z service build full tx
    let input = AddInputArgs { tx_hash, index: 0 };
    let output = AddOutputArgs {
        capacity: (100_0000_0000 - 1_0000_0000).into(),
        udt_amount: None,
    };
    let full_tx = z_service
        .add_input_and_output(open_tx, input, output)
        .unwrap();

    // signer in Z service sign the full tx
    let full_tx = z_service.signer.sign_tx(full_tx).unwrap();
    dump_data(&full_tx, "./free-space/udt_full_tx.json").unwrap();

    // commiter in Z service send tx
    z_service.committer.send_tx(full_tx).unwrap();
}

fn bob_build_signed_otx() -> Result<TxInfo> {
    // 1. init bob's wallet
    let bob_wallet = Wallet::init_account();
    let bob_otx_address = bob_wallet.get_omni_otx_address();
    let bob_omni_otx_script: Script = bob_otx_address.into();
    println!("{:?}", bob_omni_otx_script.code_hash());
    println!("{:?}", bob_omni_otx_script.args().raw_data().as_ref());
    println!("{:?}", bob_omni_otx_script.hash_type());

    // 2. transfer udt to bob omni address
    let tx_hash = prepare_udt(51u128, bob_otx_address).unwrap();
    let out_point = OutPoint::new(Byte32::from_slice(tx_hash.as_bytes())?, 0u32);
    let balance_payload = GetBalancePayload {
        item: JsonItem::OutPoint(out_point.clone().into()),
        asset_infos: HashSet::new(),
        extra: None,
        tip_block_number: None,
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    let balance = mercury_client.get_balance(balance_payload).unwrap();
    assert_eq!(balance.balances.len(), 2);
    assert_eq!(balance.balances[0].occupied, 144_0000_0000u128.into());
    assert_eq!(balance.balances[0].free, 0u128.into());
    assert_eq!(balance.balances[0].frozen, 0u128.into());
    assert_eq!(balance.balances[1].free, 51u128.into());

    let balance_payload = GetBalancePayload {
        item: JsonItem::Address(bob_otx_address.to_string()),
        asset_infos: HashSet::new(),
        extra: None,
        tip_block_number: None,
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    let balance = mercury_client.get_balance(balance_payload).unwrap();
    assert_eq!(balance.balances.len(), 2);
    assert_eq!(balance.balances[0].occupied, 144_0000_0000u128.into());
    assert_eq!(balance.balances[0].free, 0u128.into());
    assert_eq!(balance.balances[0].frozen, 0u128.into());
    assert_eq!(balance.balances[1].free, 51u128.into());

    // 3. bob generate open transaction, pay 51 UDT
    let udt_issuer_script: Script = UDT_1_HOLDER_SECP_ADDRESS.get().unwrap().into();
    let xudt_type_script = Script::new_builder()
        .code_hash(Byte32::from_slice(XUDT_DEVNET_TYPE_HASH.as_bytes()).unwrap())
        .hash_type(ScriptHashType::Type.into())
        .args(udt_issuer_script.calc_script_hash().raw_data().pack())
        .build();
    let xudt_output = CellOutput::new_builder()
        .capacity(capacity_bytes!(144).pack())
        .lock(bob_omni_otx_script)
        .type_(Some(xudt_type_script).pack())
        .build();
    let xudt_data = Bytes::from(0u128.to_le_bytes().to_vec());
    let open_tx = bob_wallet
        .gen_open_tx_pay_udt(vec![out_point], vec![xudt_output], vec![xudt_data.pack()])
        .unwrap();
    let file = "./free-space/udt_bob_otx_unsigned.json";
    dump_data(&open_tx, file).unwrap();

    // 4. bob sign the otx
    let open_tx = bob_wallet.sign_open_tx(open_tx).unwrap();
    dump_data(&open_tx, "./free-space/udt_bob_otx_signed.json").unwrap();

    Ok(open_tx)
}
