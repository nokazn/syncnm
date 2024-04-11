use std::fmt::{Debug, Display};

use data_encoding::BASE32_NOPAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{core::Result, errors::to_error};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Hash(pub String);

impl Display for Hash {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Hash {
  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

pub trait Hashable {
  fn to_hash_target(&self) -> Result<impl AsRef<[u8]>>;

  fn generate_hash(&self) -> Result<Hash> {
    let bytes = self.to_hash_target();
    match bytes {
      Ok(bytes) => {
        let mut generator = Sha256::new();
        generator.update(bytes);
        let raw_hash = generator.finalize();
        let hash = BASE32_NOPAD.encode(&raw_hash[..20]).to_lowercase();
        dbg!(&hash);
        Ok(Hash(hash))
      }
      Err(error) => Err(to_error(error)),
    }
  }
}
