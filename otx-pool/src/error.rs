use otx_format::error::{OtxError, OtxFormatError};

use anyhow::Result;
use derive_more::Display;
use jsonrpsee_core::Error;
use molecule::error::VerificationError;

use std::fmt::Debug;

pub type InnerResult<T> = Result<T, OtxPoolRpcError>;

#[derive(Debug, Display)]
pub struct OtxPoolRpcError(pub Box<dyn OtxError + Send>);

impl From<OtxPoolRpcError> for Error {
    fn from(err: OtxPoolRpcError) -> Error {
        Error::Custom(format!(
            "Error({}): {:?}",
            err.0.err_code(),
            err.0.message()
        ))
    }
}

impl From<OtxFormatError> for OtxPoolRpcError {
    fn from(err: OtxFormatError) -> Self {
        OtxPoolRpcError(Box::new(err))
    }
}

impl From<VerificationError> for OtxPoolRpcError {
    fn from(err: VerificationError) -> Self {
        OtxPoolRpcError(Box::new(err))
    }
}
