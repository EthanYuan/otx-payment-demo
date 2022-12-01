use super::super::IntegrationTest;
use crate::const_definition::CKB_URI;
use crate::const_definition::{
    GENESIS_BUILT_IN_ADDRESS_1, GENESIS_BUILT_IN_ADDRESS_1_PRIVATE_KEY, MERCURY_URI, UDT_1_HASH,
    UDT_1_HOLDER_ACP_ADDRESS, UDT_1_HOLDER_ACP_ADDRESS_PK,
};
use crate::service::{AddInputArgs, AddOutputArgs, OtxService};
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;
use crate::utils::instruction::mercury::issue_udt_1;
use crate::utils::instruction::send_transaction_to_ckb;
use crate::utils::lock::omni::{MultiSigArgs, TxInfo};
use crate::utils::mercury_client_rpc::MercuryRpcClient;
use crate::utils::signer::sign_transaction;
use crate::wallet::{GenOpenTxArgs, Wallet};

use anyhow::Result;
use ckb_sdk::{unlock::IdentityFlag, Address, HumanCapacity};
use ckb_types::H256;
use core_rpc_types::{
    AssetInfo, GetBalancePayload, JsonItem, OutputCapacityProvider, ToInfo, TransferPayload,
};

use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

inventory::submit!(IntegrationTest {
    name: "z_aggregate_otxs_omni_lock",
    test_fn: z_aggregate_otxs_omni_lock
});
fn z_aggregate_otxs_omni_lock() {
    let alice_otx = alice_build_signed_otx().unwrap();
    bob_build_signed_otx().unwrap();
    // let _carol_otx_file = carol_build_signed_otx().unwrap();

    let z_service = OtxService::new(vec![alice_otx], CKB_URI);
    let tx_hash = ckb_cli_transfer_ckb(z_service.signer.get_secp_address(), 99).unwrap();

    // builder in Z service build full tx
    let open_tx = z_service.builder.merge_otxs().unwrap();
    dump_data(&open_tx, "./free-space/usercase_otxs_merged.json").unwrap();
    let input = AddInputArgs { tx_hash, index: 0 };
    let output = AddOutputArgs {
        capacity: (99_0000_0000 + 1_0000_0000 - 10_0000).into(),
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

    // 2. transfer capacity to alice omni address
    let alice_omni_address = alice_wallet.get_omni_otx_address();
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
    let file = "./free-space/alice_otx_unsigned.json";
    dump_data(&open_tx, file).unwrap();

    // 4. alice sign the otx
    let open_tx = alice_wallet.sign_open_tx(open_tx).unwrap();
    dump_data(&open_tx, "./free-space/alice_otx_signed.json").unwrap();

    Ok(open_tx)
}

fn bob_build_signed_otx() -> Result<()> {
    // 1. init bob's wallet
    let bob_wallet = Wallet::init_account();
    let bob_otx_address = bob_wallet.get_omni_otx_address();

    // 2. transfer udt to bob
    let _tx_hash = prepare_udt(51u128, bob_otx_address).unwrap();
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
    assert_eq!(balance.balances[1].free, 51u128.into());

    // 3. bob generate open transaction, pay 51 UDT
    let _gen_open_tx_args = GenOpenTxArgs {
        omni_identity_flag: IdentityFlag::PubkeyHash,
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: bob_otx_address.to_owned(),
        capacity_with_open: Some((
            HumanCapacity::from_str("144.0").unwrap(),
            HumanCapacity::from_str("0.0").unwrap(),
        )),
        udt_amount_with_open: Some((0, 51)),
        fee_rate: 0,
    };
    // let open_tx = bob_wallet.gen_open_tx(&gen_open_tx_args).unwrap();
    // let file = "./free-space/bob_otx_unsigned.json";
    // dump_data(&open_tx, file).unwrap();

    // // 4. bob sign the otx
    // let open_tx = bob_wallet.sign_open_tx(open_tx).unwrap();
    // dump_data(&open_tx, "./free-space/bob_otx_signed.json").unwrap();

    Ok(())
}

fn _carol_build_signed_otx() -> Result<PathBuf> {
    todo!()
}

fn prepare_udt(amount: u128, to_address: &Address) -> Result<H256> {
    // prepare udt
    issue_udt_1().unwrap();
    let udt_hash = UDT_1_HASH.get().unwrap();
    let acp_address_with_udt = UDT_1_HOLDER_ACP_ADDRESS.get().unwrap();
    let acp_address_pk = UDT_1_HOLDER_ACP_ADDRESS_PK.get().unwrap();

    let payload = TransferPayload {
        asset_info: AssetInfo::new_udt(udt_hash.to_owned()),
        from: vec![
            JsonItem::Address(acp_address_with_udt.to_string()),
            JsonItem::Address(GENESIS_BUILT_IN_ADDRESS_1.to_string()),
        ],
        to: vec![ToInfo {
            address: to_address.to_string(),
            amount: amount.into(),
        }],
        output_capacity_provider: Some(OutputCapacityProvider::From),
        pay_fee: None,
        fee_rate: None,
        since: None,
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    let tx = mercury_client.build_transfer_transaction(payload).unwrap();
    let tx = sign_transaction(
        tx,
        &[
            acp_address_pk.to_owned(),
            GENESIS_BUILT_IN_ADDRESS_1_PRIVATE_KEY.to_owned(),
        ],
    )
    .unwrap();
    send_transaction_to_ckb(tx)
}
