//! Crypto helpers for hashing, signing, encryption and decryption.

#[cfg(feature = "orm")]
mod password;

#[cfg(feature = "orm")]
pub(crate) use password::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "crypto-sm")] {
        mod sm3;
        mod sm4;

        pub use sm3::{derive_key, digest};
        pub use sm4::{decrypt, encrypt};

        /// Digest type.
        pub type Digest = ::sm3::Sm3;
    } else {
        mod aes256;
        mod sha256;

        pub use aes256::{decrypt, encrypt};
        pub use sha256::{derive_key, digest};

        /// Digest type.
        pub type Digest = ::sha2::Sha256;
    }
}

mod sha1;

pub use sha1::checksum;
