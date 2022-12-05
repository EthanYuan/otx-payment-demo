use super::super::IntegrationTest;
use crate::const_definition::CKB_URI;
use crate::const_definition::{MERCURY_URI, UDT_1_HOLDER_SECP_ADDRESS, XUDT_DEVNET_TYPE_HASH};
use crate::service::{AddInputArgs, AddOutputArgs, OtxService};
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;
use crate::utils::instruction::mercury::prepare_udt;

use crate::utils::lock::omni::{MultiSigArgs, TxInfo};
use crate::utils::mercury_client_rpc::MercuryRpcClient;

use crate::wallet::{GenOpenTxArgs, Wallet};

use anyhow::Result;
use ckb_sdk::{unlock::IdentityFlag, HumanCapacity};
use ckb_types::{
    bytes::Bytes,
    core::{capacity_bytes, Capacity, ScriptHashType},
    packed::{Byte32, CellOutput, OutPoint, Script},
    prelude::*,
};
use core_rpc_types::{GetBalancePayload, JsonItem};

use std::collections::HashSet;
use std::str::FromStr;

inventory::submit!(IntegrationTest {
    name: "z_aggregate_otxs_omni_lock",
    test_fn: z_aggregate_otxs_omni_lock
});
fn z_aggregate_otxs_omni_lock() {
    // {
    //     inputs: [
    //         {capacity: 151, data: "", type: "", lock: Alice},
    //         {capacity: 144, data: 51, type: xudt z, lock: Bob},
    //         {capacity: 100, data: "", type: "", lock: Carol},
    //         {capacity: 144, data: 9, type: xudt z, lock: Carol},
    //         {capacity: 142, data: 100, type: xudt z, lock: Z}
    //     ],
    //     outputs: [
    //         {capacity: 151-51, data: "", type: "", lock: Alice},
    //         {capacity: 144, data: 51-51, type: xudt z, lock: Bob},
    //         {capacity: 100-1, data: "", type: "", lock: Carol},
    //         {capacity: 144, data: 9+1, type: xudt z, lock: Carol},
    //         {capacity: 142+50+1, data: 100+50, type: xudt z, lock: Z} ]
    // }
    let alice_otx = alice_build_signed_otx().unwrap();
    let bob_otx = bob_build_signed_otx().unwrap();
    let carol_otx_file = carol_build_signed_otx().unwrap();

    let z_service = OtxService::new(vec![alice_otx, bob_otx, carol_otx_file], CKB_URI);
    let tx_hash = prepare_udt(100u128, z_service.signer.get_secp_address()).unwrap();

    // builder in Z service build full tx
    let open_tx = z_service.builder.merge_otxs().unwrap();
    dump_data(&open_tx, "./free-space/usercase_otxs_merged.json").unwrap();
    let input = AddInputArgs { tx_hash, index: 0 };
    let output = AddOutputArgs {
        capacity: (142_0000_0000 + 50_0000_0000 + 1_0000_0000).into(),
        udt_amount: Some(100 + 50),
    };
    let full_tx = z_service
        .add_input_and_output(open_tx, input, output)
        .unwrap();

    // signer in Z service sign the full tx
    let full_tx = z_service.signer.sign_tx(full_tx).unwrap();
    dump_data(&full_tx, "./free-space/usercase_full_tx.json").unwrap();

    // commiter in Z service send tx
    z_service.committer.send_tx(full_tx).unwrap();
}

fn alice_build_signed_otx() -> Result<TxInfo> {
    // 1. init Alice's wallet instance
    let alice_wallet = Wallet::init_account();
    let alice_omni_address = alice_wallet.get_omni_otx_address();

    // 2. transfer capacity to alice omni address
    let _tx_hash = ckb_cli_transfer_ckb(alice_omni_address, 151).unwrap();
    let capacity = ckb_cli_get_capacity(alice_omni_address).unwrap();
    assert_eq!(151f64, capacity);

    // 3. alice generate open transaction, pay 51 CKB
    let gen_open_tx_args = GenOpenTxArgs {
        omni_identity_flag: IdentityFlag::PubkeyHash,
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: alice_omni_address.to_owned(),
        capacity_with_open: Some((
            HumanCapacity::from_str("100.0").unwrap(),
            HumanCapacity::from_str("51.0").unwrap(),
        )),
        udt_amount_with_open: None,
        fee_rate: 0,
    };
    let open_tx = alice_wallet.gen_open_tx(&gen_open_tx_args).unwrap();
    let file = "./free-space/usercase_alice_otx_unsigned.json";
    dump_data(&open_tx, file).unwrap();

    // 4. alice sign the otx
    let open_tx = alice_wallet.sign_open_tx(open_tx).unwrap();
    dump_data(&open_tx, "./free-space/usercase_alice_otx_signed.json").unwrap();

    Ok(open_tx)
}

fn bob_build_signed_otx() -> Result<TxInfo> {
    // 1. init bob's wallet
    let bob_wallet = Wallet::init_account();
    let bob_otx_address = bob_wallet.get_omni_otx_address();
    let bob_omni_otx_script: Script = bob_otx_address.into();

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
    let file = "./free-space/usercase_bob_otx_unsigned.json";
    dump_data(&open_tx, file).unwrap();

    // 4. bob sign the otx
    let open_tx = bob_wallet.sign_open_tx(open_tx).unwrap();
    dump_data(&open_tx, "./free-space/usercase_bob_otx_signed.json").unwrap();

    Ok(open_tx)
}

fn carol_build_signed_otx() -> Result<TxInfo> {
    // 1. init carol's wallet
    let wallet = Wallet::init_account();
    let otx_address = wallet.get_omni_otx_address();
    let omni_otx_script: Script = otx_address.into();

    // 2. transfer capacity to alice omni address
    let tx_hash = ckb_cli_transfer_ckb(otx_address, 100).unwrap();
    let capacity = ckb_cli_get_capacity(otx_address).unwrap();
    let out_point_1 = OutPoint::new(Byte32::from_slice(tx_hash.as_bytes())?, 0u32);
    assert_eq!(100f64, capacity);

    // 3. transfer udt to carol omni address
    let tx_hash = prepare_udt(9u128, otx_address).unwrap();
    let out_point_2 = OutPoint::new(Byte32::from_slice(tx_hash.as_bytes())?, 0u32);
    let balance_payload = GetBalancePayload {
        item: JsonItem::OutPoint(out_point_2.clone().into()),
        asset_infos: HashSet::new(),
        extra: None,
        tip_block_number: None,
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    let balance = mercury_client.get_balance(balance_payload).unwrap();
    assert_eq!(balance.balances.len(), 2);
    assert_eq!(balance.balances[0].occupied, 144_0000_0000u128.into());
    assert_eq!(balance.balances[1].free, 9u128.into());

    // 4. carol generate open transaction, pay 1 CKB, get 1 UDT
    let omni_output = CellOutput::new_builder()
        .capacity(capacity_bytes!(99).pack())
        .lock(omni_otx_script.clone())
        .build();
    let data = Bytes::default();

    let udt_issuer_script: Script = UDT_1_HOLDER_SECP_ADDRESS.get().unwrap().into();
    let xudt_type_script = Script::new_builder()
        .code_hash(Byte32::from_slice(XUDT_DEVNET_TYPE_HASH.as_bytes()).unwrap())
        .hash_type(ScriptHashType::Type.into())
        .args(udt_issuer_script.calc_script_hash().raw_data().pack())
        .build();
    let xudt_output = CellOutput::new_builder()
        .capacity(capacity_bytes!(144).pack())
        .lock(omni_otx_script)
        .type_(Some(xudt_type_script).pack())
        .build();
    let xudt_data = Bytes::from(10u128.to_le_bytes().to_vec());

    let open_tx = wallet
        .gen_open_tx_pay_udt(
            vec![out_point_1, out_point_2],
            vec![omni_output, xudt_output],
            vec![data.pack(), xudt_data.pack()],
        )
        .unwrap();
    let file = "./free-space/usercase_carol_otx_unsigned.json";
    dump_data(&open_tx, file).unwrap();

    let open_tx = wallet.sign_open_tx(open_tx).unwrap();
    dump_data(&open_tx, "./free-space/usercase_carol_otx_signed.json").unwrap();

    Ok(open_tx)
}
