use super::super::IntegrationTest;
use crate::utils::address::omni::{
    build_omnilock_addr_from_secp, AddInputArgs, AddOutputArgs, MultiSigArgs, OmniLockInfo,
    SignTxArgs, TxInfo,
};
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;
use crate::wallet::Wallet;
use crate::wallet::{sign_open_tx, GenOpenTxArgs};
use crate::{utils::address::secp::generate_rand_secp_address_pk_pair, wallet};

use ckb_sdk::{
    constants::SIGHASH_TYPE_HASH,
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{
        balance_tx_capacity, fill_placeholder_witnesses, omni_lock::OmniLockTransferBuilder,
        unlock_tx, CapacityBalancer, TxBuilder,
    },
    types::NetworkType,
    unlock::{
        opentx::OpentxWitness, IdentityFlag, MultisigConfig, OmniLockConfig, OmniLockScriptSigner,
        SecpSighashUnlocker,
    },
    unlock::{OmniLockUnlocker, OmniUnlockMode, ScriptUnlocker},
    util::{blake160, keccak160},
    Address, AddressPayload, HumanCapacity, ScriptGroup, ScriptId, SECP256K1,
};

use anyhow::{anyhow, Result};

use std::str::FromStr;
use std::{collections::HashMap, error::Error as StdErr, fs, path::PathBuf};

inventory::submit!(IntegrationTest {
    name: "z_aggregate_otxs_omni_lock",
    test_fn: z_aggregate_otxs_omni_lock
});
fn z_aggregate_otxs_omni_lock() {
    let _alice_otx_file = alice_build_signed_otx().unwrap();
}

fn alice_build_signed_otx() -> Result<PathBuf> {
    // 1. init Alice's wallet instance
    let alice_wallet = Wallet::init_account();

    // 2. transfer capacity to alice omni address
    let alice_omni_address = alice_wallet.get_omni_otx_address();
    let _tx_hash = ckb_cli_transfer_ckb(&alice_omni_address, 151).unwrap();
    let capacity = ckb_cli_get_capacity(&alice_omni_address).unwrap();
    assert_eq!(151 as f64, capacity);

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
