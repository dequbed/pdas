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

    pub fn parse_str(input: &str) -> crate::error::Result<UUID> {
        let u = Uuid::parse_str(input)?;
        Ok(Self::new(u))
    }
}
