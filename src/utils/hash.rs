use data_encoding::BASE32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Display};

use crate::{core::Result, errors::to_error};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Hash(pub String);

impl Display for Hash {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

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

  fn generate_hash(&self) -> Result<Hash>
  where
    Self: serde::Serialize + Debug,
  {
    let bytes = self.to_bytes();
    match bytes {
      Ok(bytes) => {
        let mut generator = Sha256::new();
        generator.update(bytes);
        let raw_hash = generator.finalize();
        let hash = BASE32.encode(&raw_hash);
        Ok(Hash(hash))
      }
      Err(error) => Err(to_error(error)),
    }
  }
}
