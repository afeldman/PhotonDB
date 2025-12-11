//! ReQL type definitions

use serde::{Deserialize, Serialize};

/// Query type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Start,
    Continue,
    Stop,
    Wait,
}

/// Response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    SuccessAtom,
    SuccessSequence,
    SuccessPartial,
    WaitComplete,
    ServerInfo,
    ClientError,
    CompileError,
    RuntimeError,
}
