use open_tx::jsonrpc_types::OpenTransaction;

use ckb_types::H256;
use dashmap::DashMap;

pub struct OtxPool {
    _otxs: DashMap<H256, OpenTransaction>,
}

impl OtxPool {
    pub fn new() -> Self {
        OtxPool {
            _otxs: DashMap::new(),
        }
    }
}

impl Default for OtxPool {
    fn default() -> Self {
        Self::new()
    }
}
