use otx_format::jsonrpc_types::OpenTransaction;

use serde_derive::{Deserialize, Serialize};

pub type Id = u64;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageFromHost {
    // Notify
    NewOtx((Id, OpenTransaction)),
    NewInterval,
    OtxPoolStart,
    OtxPoolStop,
    DeleteOtx(Id),

    // Request
    GetPluginInfo,
    // Response
}

pub enum MessageFromPlugin {
    // Notify

    // Response
    Ok,
    Error(String),
    PluginInfo(PluginInfo),

    // Request
    NewOtx(OpenTransaction),
    DiscardOtx(Id),
    ModifyOtx((Id, OpenTransaction)),
    SendCkbTx(OpenTransaction),
}
