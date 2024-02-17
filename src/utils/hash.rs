use base64ct::{Base64, Encoding};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Hash(pub String);

impl Hash {
  pub fn to_string(&self) -> &str {
    self.0.as_str()
  }
}

pub trait Hashable {
  fn to_bytes(&self) -> serde_json::Result<Vec<u8>>
  where
    Self: serde::Serialize,
  {
    let json = serde_json::to_string(self)?;
    Ok(json.into_bytes())
  }

  fn generate_hash(&self) -> serde_json::Result<Hash>
  where
    Self: serde::Serialize + Debug,
  {
    self.to_bytes().map(|bytes| {
      let mut generator = Sha256::new();
      generator.update(bytes);
      let raw_hash = generator.finalize();
      let hash = Base64::encode_string(&raw_hash);
      Hash(hash)
    })
  }
}
