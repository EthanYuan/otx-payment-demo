use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Display, Hash, PartialEq, Eq)]
pub enum OtxFormatError {
    #[display(fmt = "version {} is not supported", _0)]
    VersionNotSupported(String),

    #[display(fmt = "{} map has duplicate keypairs", _0)]
    OtxMapHasDuplicateKeypair(String),

    #[display(fmt = "map parse missing field {}", _0)]
    OtxMapParseMissingField(String),

    #[display(fmt = "map parse failed: {}", _0)]
    OtxMapParseFailed(String),
}
