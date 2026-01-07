mod aead;
mod kdf;
mod random;
mod error;

pub use aead::{encrypt, decrypt, EncryptionKey, Nonce};
pub use kdf::{derive_key, DerivedKey, KeyDerivationParams};
pub use random::{generate_key, generate_nonce, generate_salt};
pub use error::{CryptoError, Result};

// Re-export commonly used types
pub use zeroize::Zeroize;

/// Key size for ChaCha20-Poly1305 (256 bits)
pub const KEY_SIZE: usize = 32;

/// Nonce size for ChaCha20-Poly1305 (96 bits)
pub const NONCE_SIZE: usize = 12;

/// Salt size for key derivation (128 bits)
pub const SALT_SIZE: usize = 16;

/// Tag size for authentication (128 bits)
pub const TAG_SIZE: usize = 16;
