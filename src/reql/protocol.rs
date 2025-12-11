// Generated Cap'n Proto code modules - place at crate root to match capnp's expectations
// These are re-exported from lib.rs as types_capnp, query_capnp, etc.

use crate::reql::datum::Datum as RustDatum;

// Re-export generated modules from crate root
pub use crate::handshake_capnp;
pub use crate::query_capnp;
pub use crate::response_capnp;
pub use crate::term_capnp;
pub use crate::types_capnp;

// Convenience conversions between Cap'n Proto and Rust types
impl RustDatum {
    /// Convert Rust Datum to Cap'n Proto Datum
    pub fn to_capnp<'a>(&self, builder: &mut types_capnp::datum::Builder<'a>) -> capnp::Result<()> {
        match self {
            RustDatum::Null => builder.set_null(()),
            RustDatum::Boolean(b) => builder.set_bool(*b),
            RustDatum::Number(n) => builder.set_number(*n),
            RustDatum::String(s) => builder.set_string(s.as_str()),
            RustDatum::Array(arr) => {
                let mut list = builder.reborrow().init_array(arr.len() as u32);
                for (i, item) in arr.iter().enumerate() {
                    item.to_capnp(&mut list.reborrow().get(i as u32))?;
                }
            }
            RustDatum::Object(obj) => {
                let mut pairs = builder.reborrow().init_object(obj.len() as u32);
                for (i, (k, v)) in obj.iter().enumerate() {
                    let mut pair = pairs.reborrow().get(i as u32);
                    pair.set_key(k.as_str());
                    v.to_capnp(&mut pair.init_value())?;
                }
            }
        }
        Ok(())
    }

    /// Convert Cap'n Proto Datum to Rust Datum
    pub fn from_capnp(reader: types_capnp::datum::Reader<'_>) -> capnp::Result<Self> {
        use types_capnp::datum::Which;
        match reader.which()? {
            Which::Null(()) => Ok(RustDatum::Null),
            Which::Bool(b) => Ok(RustDatum::Boolean(b)),
            Which::Number(n) => Ok(RustDatum::Number(n)),
            Which::String(s) => Ok(RustDatum::String(s?.to_string()?)),
            Which::Array(arr) => {
                let arr = arr?;
                let mut result = Vec::with_capacity(arr.len() as usize);
                for item in arr.iter() {
                    result.push(Self::from_capnp(item)?);
                }
                Ok(RustDatum::Array(result))
            }
            Which::Object(obj) => {
                let obj = obj?;
                let mut result = std::collections::HashMap::new();
                for pair in obj.iter() {
                    let key = pair.get_key()?.to_string()?;
                    let value = Self::from_capnp(pair.get_value()?)?;
                    result.insert(key, value);
                }
                Ok(RustDatum::Object(result))
            }
            Which::Json(json_str) => {
                // Parse JSON string into Datum
                let json = json_str?.to_string()?;
                serde_json::from_str(&json)
                    .map_err(|e| capnp::Error::failed(format!("JSON parse error: {}", e)))
            }
        }
    }
}

/// Protocol version constants (from handshake.capnp)
pub const VERSION_V0_1: u32 = 0x3f61ba36;
pub const VERSION_V0_2: u32 = 0x723081e1;
pub const VERSION_V0_3: u32 = 0x5f75e83e;
pub const VERSION_V0_4: u32 = 0x400c2d20;
pub const VERSION_V1_0: u32 = 0x34c2bdc3;

pub const PROTOCOL_PROTOBUF: u32 = 0x271ffc41;
pub const PROTOCOL_JSON: u32 = 0x7e6970c7;

/// Helper to create a Query
pub fn create_query(
    query_type: query_capnp::QueryType,
    token: i64,
) -> capnp::message::Builder<capnp::message::HeapAllocator> {
    let mut message = capnp::message::Builder::new_default();
    {
        let mut query = message.init_root::<query_capnp::query::Builder<'_>>();
        query.set_type(query_type);
        query.set_token(token);
        query.set_accepts_r_json(false);
    }
    message
}

/// Helper to parse a Response
pub fn parse_response(
    message: &capnp::message::Reader<capnp::serialize::OwnedSegments>,
) -> capnp::Result<response_capnp::response::Reader<'_>> {
    message.get_root::<response_capnp::response::Reader<'_>>()
}
