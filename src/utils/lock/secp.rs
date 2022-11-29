use crate::utils::lock::{generate_rand_private_key, generate_secp_args_from_pk};

use ckb_sdk::{constants::SIGHASH_TYPE_HASH, types::NetworkType, Address, AddressPayload};
use ckb_types::{bytes::Bytes, core::ScriptHashType, packed, prelude::*, H256};

pub(crate) fn generate_rand_secp_address_pk_pair() -> (Address, H256) {
    // generate pubkey by privkey
    let pk = generate_rand_private_key();

    let args = generate_secp_args_from_pk(&pk).unwrap();

    // secp address
    let secp_code_hash =
        packed::Byte32::from_slice(SIGHASH_TYPE_HASH.as_bytes()).expect("impossible:");
    let payload = AddressPayload::new_full(
        ScriptHashType::Type,
        secp_code_hash,
        Bytes::from(args.as_bytes().to_owned()),
    );
    let address = Address::new(NetworkType::Testnet, payload, true);

    (address, pk)
}
