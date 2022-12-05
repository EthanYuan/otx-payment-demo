pub mod mercury;

use crate::const_definition::{
    ANYONE_CAN_PAY_DEVNET_TYPE_HASH, CELL_BASE_MATURE_EPOCH, CHEQUE_DEVNET_TYPE_HASH, CKB_URI,
    DAO_DEVNET_TYPE_HASH, GENESIS_EPOCH_LENGTH, MERCURY_URI, PW_LOCK_DEVNET_TYPE_HASH,
    RPC_TRY_COUNT, RPC_TRY_INTERVAL_SECS, SIGHASH_TYPE_HASH, XUDT_DEVNET_TYPE_HASH,
};

use crate::utils::ckb_rpc_client::CkbRpcClient;
use crate::utils::mercury_client_rpc::MercuryRpcClient;

use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::{OutputsValidator, Transaction};

use ckb_types::H256;
use common::lazy::{
    ACP_CODE_HASH, CHEQUE_CODE_HASH, DAO_CODE_HASH, PW_LOCK_CODE_HASH, SECP256K1_CODE_HASH,
    SUDT_CODE_HASH,
};

use serde::Serialize;

use std::ffi::OsStr;
use std::panic;
use std::process::{Child, Command, ExitStatus};
use std::thread::sleep;
use std::time::Duration;

pub fn run_command_spawn<I, S>(bin: &str, args: I) -> Result<Child>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
{
    let child = Command::new(bin)
        .env("RUST_BACKTRACE", "full")
        .args(args)
        .spawn()
        .expect("run command");
    Ok(child)
}

pub fn run_command_status<I, S>(bin: &str, args: I) -> Result<ExitStatus>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
{
    let status = Command::new(bin)
        .env("RUST_BACKTRACE", "full")
        .args(args)
        .status()
        .expect("run command");
    Ok(status)
}

pub fn run_command_output<I, S>(bin: &str, args: I) -> Result<(String, String)>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
{
    let output = Command::new(bin)
        .env("RUST_BACKTRACE", "full")
        .args(args)
        .output()
        .expect("run command");

    if !output.status.success() {
        Err(anyhow!(
            "{}",
            String::from_utf8_lossy(output.stderr.as_slice())
        ))
    } else {
        let stdout = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
        let stderr = String::from_utf8_lossy(output.stderr.as_slice()).to_string();
        Ok((stdout, stderr))
    }
}

pub(crate) fn setup() -> Vec<Child> {
    println!("Setup test environment...");
    let ckb = start_ckb_node();
    let (ckb, mercury) = start_mercury(ckb);
    vec![ckb, mercury]
}

pub(crate) fn start_mercury(ckb: Child) -> (Child, Child) {
    let mercury = run_command_spawn(
        "cargo",
        vec![
            "run",
            "--manifest-path",
            "mercury/Cargo.toml",
            "--",
            "-c",
            "dev_chain/devnet_config.toml",
            "run",
        ],
    );
    let mercury = if let Ok(mercury) = mercury {
        mercury
    } else {
        teardown(vec![ckb]);
        panic!("start mercury");
    };
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    for _try in 0..=RPC_TRY_COUNT {
        let resp = mercury_client.get_mercury_info();
        if resp.is_ok() {
            let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
            mercury_client.wait_sync();

            // This step is used to make mercury enter the normal serial sync loop state
            // only then can all initialization be completed
            if generate_blocks(1).is_err() {
                teardown(vec![ckb, mercury]);
                panic!("generate block when start mercury");
            }

            // init built-in script code hash
            let _ = SECP256K1_CODE_HASH.set(SIGHASH_TYPE_HASH);
            let _ = SUDT_CODE_HASH.set(XUDT_DEVNET_TYPE_HASH);
            let _ = ACP_CODE_HASH.set(ANYONE_CAN_PAY_DEVNET_TYPE_HASH);
            let _ = CHEQUE_CODE_HASH.set(CHEQUE_DEVNET_TYPE_HASH);
            let _ = DAO_CODE_HASH.set(DAO_DEVNET_TYPE_HASH);
            let _ = PW_LOCK_CODE_HASH.set(PW_LOCK_DEVNET_TYPE_HASH);

            return (ckb, mercury);
        } else {
            sleep(Duration::from_secs(RPC_TRY_INTERVAL_SECS))
        }
    }
    teardown(vec![ckb, mercury]);
    panic!("Setup test environment failed");
}

pub(crate) fn teardown(childs: Vec<Child>) {
    for mut child in childs {
        child.kill().expect("teardown failed");
    }
}

pub(crate) fn start_ckb_node() -> Child {
    let ckb = run_command_spawn(
        "ckb",
        vec!["run", "-C", "dev_chain/dev", "--skip-spec-check"],
    )
    .expect("start ckb dev chain");
    let ckb_client = CkbRpcClient::new(CKB_URI.to_string());
    for _try in 0..=RPC_TRY_COUNT {
        let resp = ckb_client.local_node_info();
        if resp.is_ok() {
            unlock_frozen_capacity_in_genesis();
            return ckb;
        } else {
            sleep(Duration::from_secs(RPC_TRY_INTERVAL_SECS))
        }
    }
    teardown(vec![ckb]);
    panic!("Setup test environment failed");
}

fn unlock_frozen_capacity_in_genesis() {
    let ckb_client = CkbRpcClient::new(CKB_URI.to_string());
    let epoch_view = ckb_client.get_current_epoch().expect("get_current_epoch");
    let current_epoch_number = epoch_view.number.value();
    if current_epoch_number < CELL_BASE_MATURE_EPOCH {
        for _ in 0..=(CELL_BASE_MATURE_EPOCH - current_epoch_number) * GENESIS_EPOCH_LENGTH {
            let ckb_client = CkbRpcClient::new(CKB_URI.to_string());
            let block_hash = ckb_client.generate_block().expect("generate block");
            println!("generate new block: {:?}", block_hash.to_string());
        }
    }
}

pub fn fast_forward_epochs(number: usize) -> Result<()> {
    generate_blocks(GENESIS_EPOCH_LENGTH as usize * number + 1)
}

pub(crate) fn generate_blocks(number: usize) -> Result<()> {
    let ckb_rpc_client = CkbRpcClient::new(CKB_URI.to_string());
    for _ in 0..number {
        let block_hash = ckb_rpc_client.generate_block()?;
        println!("generate new block: {:?}", block_hash.to_string());
    }
    Ok(())
}

pub fn aggregate_transactions_into_blocks() -> Result<()> {
    generate_blocks(3)?;
    generate_blocks(3)?;
    let mercury_client = MercuryRpcClient::new(MERCURY_URI.to_string());
    mercury_client.wait_sync();
    Ok(())
}

pub fn send_transaction_to_ckb(tx: Transaction) -> Result<H256> {
    let ckb_client = CkbRpcClient::new(CKB_URI.to_string());
    let tx_hash = ckb_client.send_transaction(tx, OutputsValidator::Passthrough)?;
    println!("send tx: 0x{}", tx_hash);
    aggregate_transactions_into_blocks()?;
    Ok(tx_hash)
}

pub fn dump_data<T>(data: &T, file_name: &str) -> Result<()>
where
    T: ?Sized + Serialize,
{
    let json_string = serde_json::to_string_pretty(data)?;
    std::fs::write(file_name, json_string).map_err(Into::into)
}
