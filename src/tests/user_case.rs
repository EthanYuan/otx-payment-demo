use super::super::IntegrationTest;
use crate::service::OtxService;
use crate::utils::address::omni::MultiSigArgs;
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;
use crate::wallet::{GenOpenTxArgs, Wallet};

use ckb_sdk::{unlock::IdentityFlag, HumanCapacity};

use anyhow::Result;

use std::path::PathBuf;
use std::str::FromStr;

inventory::submit!(IntegrationTest {
    name: "z_aggregate_otxs_omni_lock",
    test_fn: z_aggregate_otxs_omni_lock
});
fn z_aggregate_otxs_omni_lock() {
    let _alice_otx_file = alice_build_signed_otx().unwrap();
    let _bob_otx_file = bob_build_signed_otx().unwrap();
    let _carol_otx_file = carol_build_signed_otx().unwrap();

    let _z_service = OtxService::new();
}

fn alice_build_signed_otx() -> Result<PathBuf> {
    // 1. init Alice's wallet instance
    let alice_wallet = Wallet::init_account();

    // 2. transfer capacity to alice omni address
    let alice_omni_address = alice_wallet.get_omni_otx_address();
    let _tx_hash = ckb_cli_transfer_ckb(alice_omni_address, 151).unwrap();
    let capacity = ckb_cli_get_capacity(alice_omni_address).unwrap();
    assert_eq!(151f64, capacity);

    // 3. alice generate open transaction, pay 1 CKB
    let gen_open_tx_args = GenOpenTxArgs {
        omni_identity_flag: IdentityFlag::PubkeyHash,
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: alice_omni_address.to_owned(),
        capacity: HumanCapacity::from_str("100.0").unwrap(),
        open_capacity: HumanCapacity::from_str("51.0").unwrap(),
        fee_rate: 0,
    };
    let open_tx = alice_wallet.gen_open_tx(&gen_open_tx_args).unwrap();
    let file = "./free-space/alice_otx_unsigned.json";
    dump_data(&open_tx, file).unwrap();
    Ok(file.into())
}

fn bob_build_signed_otx() -> Result<PathBuf> {
    let _bob_wallet = Wallet::init_account();

    // 2. transfer udt to bob's omni address

    todo!()
}

fn carol_build_signed_otx() -> Result<PathBuf> {
    todo!()
}
