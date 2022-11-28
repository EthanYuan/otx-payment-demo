use crate::const_definition::{CKB_URI, OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX};
use crate::utils::address::secp::generate_rand_secp_address_pk_pair;
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;

use anyhow::{anyhow, Result};
use ckb_crypto::secp::Pubkey;
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types as json_types;
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

use ckb_types::h256;
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, Capacity, ScriptHashType, TransactionView},
    packed::{Byte32, CellDep, CellOutput, OutPoint, Script, Transaction, WitnessArgs},
    prelude::*,
    H160, H256,
};
use clap::Args;
use serde::{Deserialize, Serialize};

use std::str::FromStr;
use std::{collections::HashMap, error::Error as StdErr, fs, path::PathBuf};

pub struct OmniLockInfo {
    pub type_hash: H256,
    pub script_id: ScriptId,
    pub cell_dep: CellDep,
}

pub struct MultiSigArgs {
    /// Require first n signatures of corresponding pubkey
    pub require_first_n: u8,

    /// Multisig threshold
    pub threshold: u8,

    /// Normal sighash address
    pub sighash_address: Vec<Address>,
}

#[derive(Serialize, Deserialize)]
pub struct TxInfo {
    pub tx: json_types::TransactionView,
    pub omnilock_config: OmniLockConfig,
}

pub struct SignTxArgs {
    /// The sender private key (hex string)
    pub sender_key: Vec<H256>,
}

pub struct AddInputArgs {
    /// omnilock script deploy transaction hash
    pub tx_hash: H256,

    /// cell index of omnilock script deploy transaction's outputs
    pub index: usize,
}

pub struct AddOutputArgs {
    pub to_address: Address,
    /// The capacity to transfer (unit: CKB, example: 102.43)
    pub capacity: HumanCapacity,
}

pub fn build_omnilock_addr_from_secp(address: &Address) -> Result<Address, Box<dyn StdErr>> {
    let mut ckb_client = CkbRpcClient::new(CKB_URI);
    let cell = build_omnilock_cell_dep(&mut ckb_client, &OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX)?;
    let mut config = {
        let arg = H160::from_slice(&address.payload().args()).unwrap();
        OmniLockConfig::new_pubkey_hash(arg)
    };
    config.set_opentx_mode();
    let address_payload = {
        let args = config.build_args();
        AddressPayload::new_full(ScriptHashType::Type, cell.type_hash.pack(), args)
    };
    let lock_script = Script::from(&address_payload);
    let address = Address::new(NetworkType::Testnet, address_payload.clone(), true);
    let resp = serde_json::json!({
        "testnet": address.to_string(),
        "lock-arg": format!("0x{}", hex_string(address_payload.args().as_ref())),
        "lock-hash": format!("{:#x}", lock_script.calc_script_hash())
    });
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(address)
}

fn build_omnilock_cell_dep(
    ckb_client: &mut CkbRpcClient,
    tx_hash: &H256,
    index: usize,
) -> Result<OmniLockInfo, Box<dyn StdErr>> {
    let out_point_json = ckb_jsonrpc_types::OutPoint {
        tx_hash: tx_hash.clone(),
        index: ckb_jsonrpc_types::Uint32::from(index as u32),
    };
    let cell_status = ckb_client.get_live_cell(out_point_json, false)?;
    let script = Script::from(cell_status.cell.unwrap().output.type_.unwrap());

    let type_hash = script.calc_script_hash();
    let out_point = OutPoint::new(Byte32::from_slice(tx_hash.as_bytes())?, index as u32);

    let cell_dep = CellDep::new_builder().out_point(out_point).build();
    Ok(OmniLockInfo {
        type_hash: H256::from_slice(type_hash.as_slice())?,
        script_id: ScriptId::new_type(type_hash.unpack()),
        cell_dep,
    })
}
