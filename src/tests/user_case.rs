use super::super::IntegrationTest;
use crate::const_definition::CKB_URI;
use crate::service::{AddInputArgs, AddOutputArgs, OtxService};
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;
use crate::utils::lock::omni::{MultiSigArgs, TxInfo};
use crate::wallet::{GenOpenTxArgs, Wallet};

use anyhow::Result;
use ckb_sdk::{unlock::IdentityFlag, HumanCapacity};

use std::path::PathBuf;
use std::str::FromStr;

inventory::submit!(IntegrationTest {
    name: "z_aggregate_otxs_omni_lock",
    test_fn: z_aggregate_otxs_omni_lock
});
fn z_aggregate_otxs_omni_lock() {
    let alice_otx = alice_build_signed_otx().unwrap();
    // let _bob_otx_file = bob_build_signed_otx().unwrap();
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
        capacity: HumanCapacity::from_str("100.0").unwrap(),
        open_capacity: HumanCapacity::from_str("51.0").unwrap(),
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

fn bob_build_signed_otx() -> Result<PathBuf> {
    let _bob_wallet = Wallet::init_account();
    todo!()
}

fn carol_build_signed_otx() -> Result<PathBuf> {
    todo!()
}
