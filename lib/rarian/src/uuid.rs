use std::convert::TryInto;

use crate::error::{Result, Error};

use serde::{
    Deserialize,
    Serialize,
};


pub use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UUID(u128);

impl UUID {
    pub fn new(uuid: Uuid) -> Self{
        Self (uuid.as_u128())
    }
    pub fn from_u128(u: u128) -> Self {
        Self (u)
    }
    pub fn generate() -> Self {
        let u = Uuid::new_v4();
        Self::new(u)
    }
    pub fn as_uuid(self) -> Uuid {
        Uuid::from_u128(self.0)
    }
    pub fn as_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        let (int_bytes, _rest) = buf.split_at(std::mem::size_of::<u128>());
        // This can fail if for some reason entrydb keys are less than 16 bytes long.
        // In that case we don't have any idea how to handle or export that entry. Just
        // give up.
        Ok(Self::from_u128(u128::from_le_bytes(int_bytes.try_into().map_err(|_| Error::MalformedUUID)?)))
    }

    pub fn parse_str(input: &str) -> crate::error::Result<UUID> {
        let u = Uuid::parse_str(input)?;
        Ok(Self::new(u))
    }

    pub const fn encoded_size(&self) -> usize {
        16
    }
}
