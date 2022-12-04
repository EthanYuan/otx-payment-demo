use super::super::IntegrationTest;
use crate::const_definition::{CKB_URI, OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX};
use crate::utils::ckb_cli::{ckb_cli_get_capacity, ckb_cli_transfer_ckb};
use crate::utils::instruction::dump_data;
use crate::utils::lock::secp::generate_rand_secp_address_pk_pair;

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
        opentx::{assembler::assemble_new_tx, OpentxWitness},
        IdentityFlag, MultisigConfig, OmniLockConfig, OmniLockScriptSigner, SecpSighashUnlocker,
    },
    unlock::{OmniLockUnlocker, OmniUnlockMode, ScriptUnlocker},
    util::{blake160, keccak160},
    Address, AddressPayload, HumanCapacity, ScriptGroup, ScriptId, SECP256K1,
};

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

inventory::submit!(IntegrationTest {
    name: "test_sighash_open_transaction",
    test_fn: test_sighash_open_transaction
});
fn test_sighash_open_transaction() {
    // 1. Build opentx address
    let (address, address_pk) = generate_rand_secp_address_pk_pair();
    let omni_address = build_omnilock_addr_from_secp(&address).unwrap();

    // 2. Transfer capacity to omni address
    let tx_hash = ckb_cli_transfer_ckb(&omni_address, 99).unwrap();
    println!("transfer tx hash: {:?}", tx_hash);
    let capacity = ckb_cli_get_capacity(&omni_address).unwrap();
    assert_eq!(99f64, capacity);

    // 3. Generate open transaction, pay 1 CKB
    let gen_open_tx_args = GenOpenTxArgs {
        sender_key: Some(address_pk.clone()),
        ethereum_sender_key: None,
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: omni_address,
        capacity: HumanCapacity::from_str("98.0").unwrap(),
        open_capacity: HumanCapacity::from_str("1.0").unwrap(),
        fee_rate: 0,
    };
    let open_tx = gen_open_tx(&gen_open_tx_args).unwrap();
    dump_data(&open_tx, "./free-space/1_otx_sighash_unsigned.json").unwrap();

    // 4. Sign open transaction
    let sign_otx_args = SignTxArgs {
        sender_key: vec![address_pk],
    };
    let open_tx = sign_open_tx(
        sign_otx_args,
        "./free-space/1_otx_sighash_unsigned.json".into(),
    )
    .unwrap();
    dump_data(&open_tx, "./free-space/1_otx_sighash_signed.json").unwrap();

    // 5. Add input
    let (payee_address, payee_pk) = generate_rand_secp_address_pk_pair();
    let tx_hash = ckb_cli_transfer_ckb(&payee_address, 99).unwrap();
    println!("transfer tx hash: {:?}", tx_hash);
    let capacity = ckb_cli_get_capacity(&payee_address).unwrap();
    assert_eq!(99f64, capacity);
    let args = AddInputArgs { tx_hash, index: 0 };
    let open_tx = add_input(args, "./free-space/1_otx_sighash_signed.json".into()).unwrap();
    dump_data(&open_tx, "./free-space/1_otx_sighash_signed_add_input.json").unwrap();

    // 6. Add output
    let args = AddOutputArgs {
        to_address: payee_address,
        capacity: (99_0000_0000 + 1_0000_0000 - 10_0000).into(),
    };
    let open_tx = add_output(
        args,
        "./free-space/1_otx_sighash_signed_add_input.json".into(),
    )
    .unwrap();
    dump_data(
        &open_tx,
        "./free-space/1_otx_sighash_signed_add_output.json",
    )
    .unwrap();

    // 7. Sign the new input
    let args = &SignTxArgs {
        sender_key: vec![payee_pk],
    };
    let tx = sign_tx(
        args,
        "./free-space/1_otx_sighash_signed_add_output.json".into(),
    )
    .unwrap();
    dump_data(&tx, "./free-space/1_full_tx.json").unwrap();

    // 8. Send the tx
    let tx_hash = send_tx("./free-space/1_full_tx.json".into()).unwrap();
    println!("tx_hash: {:?}", tx_hash);
}

inventory::submit!(IntegrationTest {
    name: "test_ethereum_open_transaction",
    test_fn: test_ethereum_open_transaction
});
fn test_ethereum_open_transaction() {
    // 1. Build opentx address
    let (_secp_address, pk) = generate_rand_secp_address_pk_pair();
    let omni_address = build_omnilock_addr_flag_ethereum_from_pk(&pk).unwrap();

    // 2. Transfer capacity to omni address
    let tx_hash = ckb_cli_transfer_ckb(&omni_address, 99).unwrap();
    println!("transfer tx hash: {:?}", tx_hash);
    let capacity = ckb_cli_get_capacity(&omni_address).unwrap();
    assert_eq!(99f64, capacity);

    // 3. Generate open transaction, pay 1 CKB
    let gen_open_tx_args = GenOpenTxArgs {
        sender_key: None,
        ethereum_sender_key: Some(pk.clone()),
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: omni_address,
        capacity: HumanCapacity::from_str("98.0").unwrap(),
        open_capacity: HumanCapacity::from_str("1.0").unwrap(),
        fee_rate: 0,
    };
    let open_tx = gen_open_tx(&gen_open_tx_args).unwrap();
    dump_data(&open_tx, "./free-space/2_otx_ethereum_unsigned.json").unwrap();

    // 4. Sign open transaction
    let sign_otx_args = SignTxArgs {
        sender_key: vec![pk],
    };
    let open_tx = sign_open_tx(
        sign_otx_args,
        "./free-space/2_otx_ethereum_unsigned.json".into(),
    )
    .unwrap();
    dump_data(&open_tx, "./free-space/2_otx_ethereum_signed.json").unwrap();

    // 5. Add input
    let (payee_address, payee_pk) = generate_rand_secp_address_pk_pair();
    let tx_hash = ckb_cli_transfer_ckb(&payee_address, 99).unwrap();
    let args = AddInputArgs { tx_hash, index: 0 };
    let open_tx = add_input(args, "./free-space/2_otx_ethereum_signed.json".into()).unwrap();
    dump_data(
        &open_tx,
        "./free-space/2_otx_ethereum_signed_add_input.json",
    )
    .unwrap();

    // 6. Add output
    let args = AddOutputArgs {
        to_address: payee_address,
        capacity: (99_0000_0000 + 1_0000_0000 - 10_0000).into(),
    };
    let open_tx = add_output(
        args,
        "./free-space/2_otx_ethereum_signed_add_input.json".into(),
    )
    .unwrap();
    dump_data(
        &open_tx,
        "./free-space/2_otx_ethereum_signed_add_output.json",
    )
    .unwrap();

    // 7. Sign the new input
    let args = &SignTxArgs {
        sender_key: vec![payee_pk],
    };
    let tx = sign_tx(
        args,
        "./free-space/2_otx_ethereum_signed_add_output.json".into(),
    )
    .unwrap();
    dump_data(&tx, "./free-space/2_full_tx.json").unwrap();

    // 8. Send the tx
    let tx_hash = send_tx("./free-space/2_full_tx.json".into()).unwrap();
    println!("tx_hash: {:?}", tx_hash);
}

inventory::submit!(IntegrationTest {
    name: "test_merge_multiple_otxs",
    test_fn: test_merge_multiple_otxs
});
fn test_merge_multiple_otxs() {
    // 1. Build opentx 1
    let (address, address_pk) = generate_rand_secp_address_pk_pair();
    let omni_address = build_omnilock_addr_from_secp(&address).unwrap();
    let _tx_hash = ckb_cli_transfer_ckb(&omni_address, 98).unwrap();
    let gen_open_tx_args = GenOpenTxArgs {
        sender_key: Some(address_pk.clone()),
        ethereum_sender_key: None,
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: omni_address,
        capacity: HumanCapacity::from_str("97.0").unwrap(),
        open_capacity: HumanCapacity::from_str("1.0").unwrap(),
        fee_rate: 0,
    };
    let open_tx = gen_open_tx(&gen_open_tx_args).unwrap();
    dump_data(&open_tx, "./free-space/3_otx_sighash.json").unwrap();
    let sign_otx_args = SignTxArgs {
        sender_key: vec![address_pk],
    };
    let open_tx = sign_open_tx(sign_otx_args, "./free-space/3_otx_sighash.json".into()).unwrap();
    dump_data(&open_tx, "./free-space/3_otx_sighash.json").unwrap();

    // 2. Build opentx 2
    let (_secp_address, pk) = generate_rand_secp_address_pk_pair();
    let omni_address = build_omnilock_addr_flag_ethereum_from_pk(&pk).unwrap();
    let _tx_hash = ckb_cli_transfer_ckb(&omni_address, 98).unwrap();
    let gen_open_tx_args = GenOpenTxArgs {
        sender_key: None,
        ethereum_sender_key: Some(pk.clone()),
        multis_args: MultiSigArgs {
            require_first_n: 1,
            threshold: 1,
            sighash_address: vec![],
        },
        receiver: omni_address,
        capacity: HumanCapacity::from_str("97.0").unwrap(),
        open_capacity: HumanCapacity::from_str("1.0").unwrap(),
        fee_rate: 0,
    };
    let open_tx = gen_open_tx(&gen_open_tx_args).unwrap();
    dump_data(&open_tx, "./free-space/3_otx_ethereum.json").unwrap();
    let sign_otx_args = SignTxArgs {
        sender_key: vec![pk],
    };
    let open_tx = sign_open_tx(sign_otx_args, "./free-space/3_otx_ethereum.json".into()).unwrap();
    dump_data(&open_tx, "./free-space/3_otx_ethereum.json").unwrap();

    // 3 merge otxs
    let args = MergeOpenTxArgs {
        in_tx_file: vec![
            "./free-space/3_otx_sighash.json".into(),
            "./free-space/3_otx_ethereum.json".into(),
        ],
    };
    let open_tx = merge_otxs(args).unwrap();
    dump_data(&open_tx, "./free-space/3_full_tx.json").unwrap();

    // 4 add input and output and sign
    let (payee_address, payee_pk) = generate_rand_secp_address_pk_pair();
    let tx_hash = ckb_cli_transfer_ckb(&payee_address, 99).unwrap();
    let args = AddInputArgs { tx_hash, index: 0 };
    let open_tx = add_input(args, "./free-space/3_full_tx.json".into()).unwrap();
    dump_data(&open_tx, "./free-space/3_full_tx.json").unwrap();
    let args = AddOutputArgs {
        to_address: payee_address,
        capacity: (99_0000_0000 + 2_0000_0000 - 10_0000).into(),
    };
    let open_tx = add_output(args, "./free-space/3_full_tx.json".into()).unwrap();
    dump_data(&open_tx, "./free-space/3_full_tx.json").unwrap();
    let args = &SignTxArgs {
        sender_key: vec![payee_pk],
    };
    let tx = sign_tx(args, "./free-space/3_full_tx.json".into()).unwrap();
    dump_data(&tx, "./free-space/3_full_tx.json").unwrap();

    // 5. Send the tx
    let tx_hash = send_tx("./free-space/3_full_tx.json".into()).unwrap();
    println!("tx_hash: {:?}", tx_hash);
}

struct OmniLockInfo {
    type_hash: H256,
    script_id: ScriptId,
    cell_dep: CellDep,
}

#[derive(Args)]
struct MultiSigArgs {
    /// Require first n signatures of corresponding pubkey
    #[clap(long, value_name = "NUM", default_value = "1")]
    require_first_n: u8,

    /// Multisig threshold
    #[clap(long, value_name = "NUM", default_value = "1")]
    threshold: u8,

    /// Normal sighash address
    #[clap(long, value_name = "ADDRESS")]
    sighash_address: Vec<Address>,
}

#[derive(Serialize, Deserialize)]
struct TxInfo {
    tx: json_types::TransactionView,
    omnilock_config: OmniLockConfig,
}

#[derive(Args)]
struct GenOpenTxArgs {
    /// The sender private key (hex string)
    #[clap(long, value_name = "KEY")]
    sender_key: Option<H256>,
    /// The sender private key (hex string)
    #[clap(long, value_name = "KEY")]
    ethereum_sender_key: Option<H256>,

    #[clap(flatten)]
    multis_args: MultiSigArgs,

    /// The receiver address
    #[clap(long, value_name = "ADDRESS")]
    receiver: Address,

    /// The capacity to transfer (unit: CKB, example: 102.43)
    #[clap(long, value_name = "CKB")]
    capacity: HumanCapacity,
    /// The open transaction capacity not decided to whom (unit: CKB, example: 102.43)
    #[clap(long, value_name = "CKB")]
    open_capacity: HumanCapacity,
    #[clap(long, value_name = "NUMBER", default_value = "0")]
    fee_rate: u64,
}

#[derive(Args)]
struct SignTxArgs {
    /// The sender private key (hex string)
    #[clap(long, value_name = "KEY")]
    sender_key: Vec<H256>,
}

#[derive(Args)]
struct AddInputArgs {
    /// omnilock script deploy transaction hash
    #[clap(long, value_name = "H256")]
    tx_hash: H256,

    /// cell index of omnilock script deploy transaction's outputs
    #[clap(long, value_name = "NUMBER")]
    index: usize,
}

#[derive(Args)]
struct AddOutputArgs {
    #[clap(long, value_name = "ADDRESS")]
    to_address: Address,
    /// The capacity to transfer (unit: CKB, example: 102.43)
    #[clap(long, value_name = "CKB")]
    capacity: HumanCapacity,
}

#[derive(Args)]
struct MergeOpenTxArgs {
    /// The output transaction info file (.json)
    #[clap(long, value_name = "PATH")]
    in_tx_file: Vec<PathBuf>,
}

fn build_omnilock_addr_from_secp(address: &Address) -> Result<Address, Box<dyn StdErr>> {
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

fn build_omnilock_addr_flag_ethereum_from_pk(pk: &H256) -> Result<Address, Box<dyn StdErr>> {
    let mut ckb_client = CkbRpcClient::new(CKB_URI);
    let cell = build_omnilock_cell_dep(&mut ckb_client, &OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX)?;
    let mut config = {
        let privkey = secp256k1::SecretKey::from_slice(pk.as_bytes()).unwrap();
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);
        // println!("pubkey:{:?}", hex_string(&pubkey.serialize()));
        // println!("pubkey:{:?}", hex_string(&pubkey.serialize_uncompressed()));
        let addr = keccak160(Pubkey::from(pubkey).as_ref());
        OmniLockConfig::new_ethereum(addr)
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
) -> Result<OmniLockInfo> {
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

fn gen_open_tx(args: &GenOpenTxArgs) -> Result<TxInfo, Box<dyn StdErr>> {
    let (tx, omnilock_config) = build_open_tx(args)?;
    let tx_info = TxInfo {
        tx: json_types::TransactionView::from(tx),
        omnilock_config,
    };
    Ok(tx_info)
}

fn build_open_tx(
    args: &GenOpenTxArgs,
) -> Result<(TransactionView, OmniLockConfig), Box<dyn StdErr>> {
    let mut ckb_client = CkbRpcClient::new(CKB_URI);
    let cell = build_omnilock_cell_dep(&mut ckb_client, &OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX)?;

    let mut omnilock_config = if let Some(sender_key) = args.sender_key.as_ref() {
        let sender_key = secp256k1::SecretKey::from_slice(sender_key.as_bytes())
            .map_err(|err| format!("invalid sender secret key: {}", err))?;
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let pubkey_hash = blake160(&pubkey.serialize());
        OmniLockConfig::new_pubkey_hash(pubkey_hash)
    } else if let Some(sender_key) = args.ethereum_sender_key.as_ref() {
        let sender_key = secp256k1::SecretKey::from_slice(sender_key.as_bytes())
            .map_err(|err| format!("invalid sender secret key: {}", err))?;
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        println!("pubkey:{:?}", hex_string(&pubkey.serialize()));
        println!("pubkey:{:?}", hex_string(&pubkey.serialize_uncompressed()));
        let addr = keccak160(Pubkey::from(pubkey).as_ref());
        OmniLockConfig::new_ethereum(addr)
    } else if !args.multis_args.sighash_address.is_empty() {
        let args = &args.multis_args;
        let multisig_config =
            build_multisig_config(&args.sighash_address, args.require_first_n, args.threshold)?;
        OmniLockConfig::new_multisig(multisig_config)
    } else {
        return Err("must provide a sender-key or an ethereum-sender-key".into());
    };
    omnilock_config.set_opentx_mode();
    // Build CapacityBalancer
    let sender = Script::new_builder()
        .code_hash(cell.type_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(omnilock_config.build_args().pack())
        .build();
    let placeholder_witness = omnilock_config.placeholder_witness(OmniUnlockMode::Normal)?;
    let balancer = CapacityBalancer::new_simple(sender.clone(), placeholder_witness, args.fee_rate);

    // Build:
    //   * CellDepResolver
    //   * HeaderDepResolver
    //   * CellCollector
    //   * TransactionDependencyProvider
    let mut ckb_client = CkbRpcClient::new(CKB_URI);
    let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
    let genesis_block = BlockView::from(genesis_block);
    let mut cell_dep_resolver = DefaultCellDepResolver::from_genesis(&genesis_block)?;
    cell_dep_resolver.insert(cell.script_id, cell.cell_dep, "Omni Lock".to_string());
    let header_dep_resolver = DefaultHeaderDepResolver::new(CKB_URI);
    let mut cell_collector = DefaultCellCollector::new(CKB_URI);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(CKB_URI, 10);

    // Build base transaction
    let unlockers = build_omnilock_unlockers(Vec::new(), omnilock_config.clone(), cell.type_hash);
    let output = CellOutput::new_builder()
        .lock(sender.clone())
        .capacity(args.capacity.0.pack())
        .build();

    let builder = OmniLockTransferBuilder::new_open(
        args.open_capacity,
        vec![(output, Bytes::default())],
        omnilock_config.clone(),
        None,
    );

    let base_tx = builder.build_base(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
    )?;

    let secp256k1_data_dep = {
        // pub const SECP256K1_DATA_OUTPUT_LOC: (usize, usize) = (0, 3);
        let tx_hash = genesis_block.transactions()[0].hash();
        let out_point = OutPoint::new(tx_hash, 3u32);
        CellDep::new_builder().out_point(out_point).build()
    };

    let base_tx = base_tx
        .as_advanced_builder()
        .cell_dep(secp256k1_data_dep)
        .build();
    let (tx, _) = fill_placeholder_witnesses(base_tx, &tx_dep_provider, &unlockers)?;

    let tx = balance_tx_capacity(
        &tx,
        &balancer,
        &mut cell_collector,
        &tx_dep_provider,
        &cell_dep_resolver,
        &header_dep_resolver,
    )?;

    let tx = OmniLockTransferBuilder::remove_open_out(tx);
    let wit = OpentxWitness::new_sig_all_relative(&tx, Some(0xdeadbeef)).unwrap();
    omnilock_config.set_opentx_input(wit);
    let tx = OmniLockTransferBuilder::update_opentx_witness(
        tx,
        &omnilock_config,
        OmniUnlockMode::Normal,
        &tx_dep_provider,
        &sender,
    )?;
    Ok((tx, omnilock_config))
}

fn build_omnilock_unlockers(
    keys: Vec<secp256k1::SecretKey>,
    config: OmniLockConfig,
    omni_lock_type_hash: H256,
) -> HashMap<ScriptId, Box<dyn ScriptUnlocker>> {
    let signer = match config.id().flag() {
        IdentityFlag::PubkeyHash => SecpCkbRawKeySigner::new_with_secret_keys(keys),
        IdentityFlag::Ethereum => SecpCkbRawKeySigner::new_with_ethereum_secret_keys(keys),
        IdentityFlag::Multisig => SecpCkbRawKeySigner::new_with_secret_keys(keys),
        _ => unreachable!("should not reach here!"),
    };
    let omnilock_signer =
        OmniLockScriptSigner::new(Box::new(signer), config.clone(), OmniUnlockMode::Normal);
    let omnilock_unlocker = OmniLockUnlocker::new(omnilock_signer, config);
    let omnilock_script_id = ScriptId::new_type(omni_lock_type_hash);
    HashMap::from([(
        omnilock_script_id,
        Box::new(omnilock_unlocker) as Box<dyn ScriptUnlocker>,
    )])
}

fn build_multisig_config(
    sighash_address: &[Address],
    require_first_n: u8,
    threshold: u8,
) -> Result<MultisigConfig, Box<dyn StdErr>> {
    if sighash_address.is_empty() {
        return Err("Must have at least one sighash_address".to_string().into());
    }
    let mut sighash_addresses = Vec::with_capacity(sighash_address.len());
    for addr in sighash_address {
        let lock_args = addr.payload().args();
        if addr.payload().code_hash(None).as_slice() != SIGHASH_TYPE_HASH.as_bytes()
            || addr.payload().hash_type() != ScriptHashType::Type
            || lock_args.len() != 20
        {
            return Err(format!("sighash_address {} is not sighash address", addr).into());
        }
        sighash_addresses.push(H160::from_slice(lock_args.as_ref()).unwrap());
    }
    Ok(MultisigConfig::new_with(
        sighash_addresses,
        require_first_n,
        threshold,
    )?)
}

fn sign_open_tx(args: SignTxArgs, path: PathBuf) -> Result<TxInfo> {
    let tx_info: TxInfo = serde_json::from_slice(&fs::read(&path)?)?;
    let tx = Transaction::from(tx_info.tx.inner).into_view();
    let keys: Vec<secp256k1::SecretKey> = args
        .sender_key
        .iter()
        .map(|sender_key| {
            secp256k1::SecretKey::from_slice(sender_key.as_bytes())
                .map_err(|err| format!("invalid sender secret key: {}", err))
                .unwrap()
        })
        .collect();
    if tx_info.omnilock_config.is_pubkey_hash() || tx_info.omnilock_config.is_ethereum() {
        for (i, key) in keys.iter().enumerate() {
            let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, key);
            let hash160 = match tx_info.omnilock_config.id().flag() {
                IdentityFlag::PubkeyHash => blake2b_256(&pubkey.serialize()[..])[0..20].to_vec(),
                IdentityFlag::Ethereum => {
                    keccak160(Pubkey::from(pubkey).as_ref()).as_bytes().to_vec()
                }
                _ => unreachable!(),
            };
            if tx_info.omnilock_config.id().auth_content().as_bytes() != hash160 {
                return Err(anyhow!(
                    "key {:#x} is not in omnilock config",
                    args.sender_key[i]
                ));
            }
        }
    }
    let (tx, _) = sign_otx(&args, tx, &tx_info.omnilock_config, keys)?;
    let witness_args = WitnessArgs::from_slice(tx.witnesses().get(0).unwrap().raw_data().as_ref())?;
    let lock_field = witness_args.lock().to_opt().unwrap().raw_data();
    if lock_field != tx_info.omnilock_config.zero_lock(OmniUnlockMode::Normal)? {
        println!("> transaction has been signed!");
    } else {
        println!("failed to sign tx");
    }
    let tx_info = TxInfo {
        tx: json_types::TransactionView::from(tx),
        omnilock_config: tx_info.omnilock_config,
    };
    Ok(tx_info)
}

fn sign_otx(
    _args: &SignTxArgs,
    mut tx: TransactionView,
    omnilock_config: &OmniLockConfig,
    keys: Vec<secp256k1::SecretKey>,
) -> Result<(TransactionView, Vec<ScriptGroup>)> {
    // Unlock transaction
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(CKB_URI, 10);

    let mut ckb_client = CkbRpcClient::new(CKB_URI);
    let cell = build_omnilock_cell_dep(&mut ckb_client, &OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX)?;

    let mut _still_locked_groups = None;
    let unlockers = build_omnilock_unlockers(keys, omnilock_config.clone(), cell.type_hash);
    let (new_tx, new_still_locked_groups) = unlock_tx(tx.clone(), &tx_dep_provider, &unlockers)?;
    tx = new_tx;
    _still_locked_groups = Some(new_still_locked_groups);
    Ok((tx, _still_locked_groups.unwrap_or_default()))
}

fn add_input(args: AddInputArgs, path: PathBuf) -> Result<TxInfo> {
    let tx_info: TxInfo = serde_json::from_slice(&fs::read(path)?)?;
    // println!("> tx: {}", serde_json::to_string_pretty(&tx_info.tx)?);
    let tx = Transaction::from(tx_info.tx.inner).into_view();
    let tx = add_live_cell(&args, tx)?;
    let tx_info = TxInfo {
        tx: json_types::TransactionView::from(tx),
        omnilock_config: tx_info.omnilock_config,
    };
    Ok(tx_info)
}

fn add_live_cell(args: &AddInputArgs, tx: TransactionView) -> Result<TransactionView> {
    let mut ckb_client = CkbRpcClient::new(CKB_URI);
    let out_point_json = ckb_jsonrpc_types::OutPoint {
        tx_hash: args.tx_hash.clone(),
        index: ckb_jsonrpc_types::Uint32::from(args.index as u32),
    };
    let cell_with_status = ckb_client.get_live_cell(out_point_json, false)?;
    let input_outpoint = OutPoint::new(
        Byte32::from_slice(args.tx_hash.as_bytes())?,
        args.index as u32,
    );
    // since value should be provided in args
    let input = ckb_types::packed::CellInput::new(input_outpoint, 0);
    let cell_dep_resolver = {
        let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
        DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block))?
    };
    let code_hash = cell_with_status.cell.unwrap().output.lock.code_hash;
    let script_id = ScriptId::new_type(code_hash);
    let dep = cell_dep_resolver
        .get(&script_id)
        .as_ref()
        .unwrap()
        .0
        .clone();

    Ok(tx.as_advanced_builder().input(input).cell_dep(dep).build())
}

fn add_output(args: AddOutputArgs, path: PathBuf) -> Result<TxInfo> {
    let tx_info: TxInfo = serde_json::from_slice(&fs::read(&path)?)?;
    // println!("> tx: {}", serde_json::to_string_pretty(&tx_info.tx)?);
    let tx = Transaction::from(tx_info.tx.inner).into_view();
    let lock_script = Script::from(args.to_address.payload());
    let output = CellOutput::new_builder()
        .capacity(Capacity::shannons(args.capacity.0).pack())
        .lock(lock_script)
        .build();
    let tx = tx
        .as_advanced_builder()
        .output(output)
        .output_data(Bytes::default().pack())
        .build();
    let tx_info = TxInfo {
        tx: json_types::TransactionView::from(tx),
        omnilock_config: tx_info.omnilock_config,
    };
    Ok(tx_info)
}

fn sign_tx(args: &SignTxArgs, path: PathBuf) -> Result<TxInfo> {
    let tx_info: TxInfo = serde_json::from_slice(&fs::read(&path)?)?;
    let tx = Transaction::from(tx_info.tx.inner).into_view();
    let (tx, _) = sighash_sign(args, tx)?;
    let witness_args = WitnessArgs::from_slice(tx.witnesses().get(0).unwrap().raw_data().as_ref())?;
    let lock_field = witness_args.lock().to_opt().unwrap().raw_data();
    if lock_field != tx_info.omnilock_config.zero_lock(OmniUnlockMode::Normal)? {
        println!("> transaction ready to send!");
    } else {
        println!("failed to sign tx");
    }
    let tx_info = TxInfo {
        tx: json_types::TransactionView::from(tx),
        omnilock_config: tx_info.omnilock_config,
    };
    Ok(tx_info)
}

fn sighash_sign(
    args: &SignTxArgs,
    tx: TransactionView,
) -> Result<(TransactionView, Vec<ScriptGroup>)> {
    if args.sender_key.is_empty() {
        return Err(anyhow!("must provide sender-key to sign"));
    }
    let sender_key = secp256k1::SecretKey::from_slice(args.sender_key[0].as_bytes())
        .map_err(|err| anyhow!("invalid sender secret key: {}", err))?;
    // Build ScriptUnlocker
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![sender_key]);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let sighash_script_id = ScriptId::new_type(SIGHASH_TYPE_HASH.clone());
    let mut unlockers = HashMap::default();
    unlockers.insert(
        sighash_script_id,
        Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>,
    );

    // Build the transaction
    // let output = CellOutput::new_builder()
    //     .lock(Script::from(&args.receiver))
    //     .capacity(args.capacity.0.pack())
    //     .build();
    // let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);
    // let (tx, still_locked_groups) = builder.build_unlocked(
    //     &mut cell_collector,
    //     &cell_dep_resolver,
    //     &header_dep_resolver,
    //     &tx_dep_provider,
    //     &balancer,
    //     &unlockers,
    // )?;

    let tx_dep_provider = DefaultTransactionDependencyProvider::new(CKB_URI, 10);
    let (new_tx, new_still_locked_groups) = unlock_tx(tx, &tx_dep_provider, &unlockers)?;
    Ok((new_tx, new_still_locked_groups))
}

fn send_tx(path: PathBuf) -> Result<H256> {
    let tx_info: TxInfo = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let outputs_validator = Some(json_types::OutputsValidator::Passthrough);
    CkbRpcClient::new(CKB_URI)
        .send_transaction(tx_info.tx.inner, outputs_validator)
        .map_err(|e| anyhow!(e.to_string()))
}

fn merge_otxs(args: MergeOpenTxArgs) -> Result<TxInfo> {
    let mut txes = vec![];
    let mut omnilock_config = None;
    for in_tx in &args.in_tx_file {
        let tx_info: TxInfo = serde_json::from_slice(&fs::read(in_tx)?)?;
        // println!("> tx: {}", serde_json::to_string_pretty(&tx_info.tx)?);
        let tx = Transaction::from(tx_info.tx.inner).into_view();
        txes.push(tx);
        omnilock_config = Some(tx_info.omnilock_config);
    }
    if !txes.is_empty() {
        let mut ckb_client = CkbRpcClient::new(CKB_URI);
        let cell =
            build_omnilock_cell_dep(&mut ckb_client, &OMNI_OPENTX_TX_HASH, OMNI_OPENTX_TX_IDX)?;
        let tx_dep_provider = DefaultTransactionDependencyProvider::new(CKB_URI, 10);
        let tx = assemble_new_tx(txes, &tx_dep_provider, cell.type_hash.pack())?;
        let tx_info = TxInfo {
            tx: json_types::TransactionView::from(tx),
            omnilock_config: omnilock_config.unwrap(),
        };
        return Ok(tx_info);
    }
    Err(anyhow!("merge otxs failed!"))
}
