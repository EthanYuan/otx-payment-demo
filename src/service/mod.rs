use ckb_sdk::{Address, HumanCapacity};

use ckb_types::H256;

pub struct OtxService {}

impl OtxService {
    pub fn new() -> Self {
        OtxService {}
    }
}

impl Default for OtxService {
    fn default() -> Self {
        Self::new()
    }
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
