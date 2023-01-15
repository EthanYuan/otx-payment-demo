use otx_format::error::{OtxError, OtxFormatError};

use anyhow::Result;
use derive_more::Display;
use jsonrpsee_core::Error;
use molecule::error::VerificationError;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;

pub type InnerResult<T> = Result<T, OtxRpcError>;

#[derive(Debug, Display)]
pub struct OtxRpcError(pub Box<dyn OtxError + Send>);

impl From<OtxRpcError> for Error {
    fn from(err: OtxRpcError) -> Error {
        Error::Custom(format!(
            "Error({}): {:?}",
            err.0.err_code(),
            err.0.message()
        ))
    }
}

impl From<OtxFormatError> for OtxRpcError {
    fn from(err: OtxFormatError) -> Self {
        OtxRpcError(Box::new(err))
    }
}

impl From<VerificationError> for OtxRpcError {
    fn from(err: VerificationError) -> Self {
        OtxRpcError(Box::new(err))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Display, Hash, PartialEq, Eq)]
pub enum OtxPoolError {
    #[display(fmt = "Otx already exists")]
    OtxAlreadyExists,
}

impl OtxError for OtxPoolError {
    fn err_code(&self) -> i32 {
        match self {
            OtxPoolError::OtxAlreadyExists => -13100,
        }
    }

    fn message(&self) -> String {
        self.to_string()
    }
}
