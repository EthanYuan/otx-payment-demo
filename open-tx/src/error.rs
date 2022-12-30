use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Display, Hash, PartialEq, Eq)]
pub enum OtxError {
    #[display(fmt = "version {} is not supported", _0)]
    VersionNotSupported(String),
}
