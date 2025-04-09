use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A cryptographic key pair for a wallet.
///
/// Contains the private and public keys as hex-encoded strings.
#[derive(Serialize, Deserialize, Clone)]
pub struct KeyPair {
    /// The private key used for signing transactions (hex-encoded)
    pub private_key: String,
    /// The public key used as the wallet address (hex-encoded)
    pub public_key: String,
}

/// Collection of wallets stored by name.
///
/// Maps wallet names to their corresponding key pairs.
#[derive(Serialize, Deserialize, Default)]
pub struct Wallets {
    /// Map of wallet names to key pairs
    pub wallets: HashMap<String, KeyPair>,
}
