use base64ct::{Base64, Encoding};
use sha2::{Digest, Sha256};
use std::fmt::Debug;

pub trait Hashable {
  fn to_bytes(&self) -> serde_json::Result<Vec<u8>>
  where
    Self: serde::Serialize,
  {
    let json = serde_json::to_string(self)?;
    Ok(json.into_bytes())
  }

  fn generate_hash(&self) -> serde_json::Result<String>
  where
    Self: serde::Serialize + Debug,
  {
    self.to_bytes().map(|bytes| {
      let mut generator = Sha256::new();
      generator.update(bytes);
      let raw_hash = generator.finalize();
      let hash = Base64::encode_string(&raw_hash);
      hash
    })
  }
}
