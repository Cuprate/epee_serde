mod de;
mod error;
mod marker;
mod read_ext;
mod ser;
mod value;
mod varint;
use marker::Marker;

pub use error::{Error, Result};
pub use value::Value;

use std::io::Read;

use read_ext::ReadExt;
use ser::Serializer;

use de::Deserializer;
use serde::{de::DeserializeOwned, Serialize};

const MAX_STRING_LEN_POSSIBLE: usize = 2000000000;
const HEADER: &[u8] = b"\x01\x11\x01\x01\x01\x01\x02\x01";
const PORTABLE_STORAGE_VERSION: u8 = 1;

pub fn to_bytes<T>(object: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut buffer = Vec::new();
    buffer.extend_from_slice(HEADER);
    buffer.push(PORTABLE_STORAGE_VERSION);

    let mut serializer = Serializer::new_root(&mut buffer);
    object.serialize(&mut serializer)?;

    Ok(buffer)
}

pub fn from_bytes<T, B>(bytes: B) -> Result<T>
where
    T: DeserializeOwned,
    B: AsRef<[u8]>,
{
    let mut bytes = bytes.as_ref();

    let mut header = [0u8; 8];
    bytes.read_exact(&mut header)?;

    if header != HEADER {
        return Err(Error::MissingHeader);
    }

    let version = bytes.read_u8()?;
    if version != PORTABLE_STORAGE_VERSION {
        return Err(Error::InvalidVersion(version));
    }

    let mut deserializer = Deserializer::from_bytes(&mut bytes);

    T::deserialize(&mut deserializer)
}
