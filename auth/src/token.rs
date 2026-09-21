use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

const TOKEN_BYTES: usize = 32;
const ENCODED_TOKEN_LENGTH: usize = 43;

// No Debug or Serialize: the cookie value is a credential.
pub struct SessionToken {
    value: String,
    hash: [u8; TOKEN_BYTES],
}

impl SessionToken {
    pub fn generate() -> Result<Self, getrandom::Error> {
        let mut raw = [0_u8; TOKEN_BYTES];
        getrandom::fill(&mut raw)?;
        Ok(Self {
            value: URL_SAFE_NO_PAD.encode(raw),
            hash: Sha256::digest(raw).into(),
        })
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// Persist this digest, never `value()`. Matches the BYTEA(32-byte) constraint.
    pub fn hash(&self) -> &[u8; TOKEN_BYTES] {
        &self.hash
    }

    /// Reject malformed cookies before querying PostgreSQL.
    pub fn hash_cookie(value: &str) -> Option<[u8; TOKEN_BYTES]> {
        if value.len() != ENCODED_TOKEN_LENGTH {
            return None;
        }
        let raw: [u8; TOKEN_BYTES] = URL_SAFE_NO_PAD.decode(value).ok()?.try_into().ok()?;
        Some(Sha256::digest(raw).into())
    }
}
