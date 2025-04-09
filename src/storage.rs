use crate::errors::{Result, WalletError};
use crate::models::{KeyPair, Wallets};
use secp256k1::PublicKey;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub const WALLET_DIR: &str = ".wallets";
const WALLET_FILE: &str = "wallets.json";

impl Wallets {
    /// Loads wallet data from local storage.
    ///
    /// Attempts to read wallet data from the wallet file in the .wallets directory.
    /// If the directory or file doesn't exist, it creates an empty wallets collection.
    ///
    /// # Returns
    ///
    /// * `Ok(Wallets)` - The loaded wallets collection
    /// * `Err(WalletError)` - If an error occurs while reading or parsing wallet data
    pub fn load() -> Result<Self> {
        // Create wallet directory if it doesn't exist
        let wallet_path = Path::new(WALLET_DIR);
        if !wallet_path.exists() {
            fs::create_dir_all(wallet_path).map_err(|e| WalletError::StorageCreate {
                path: WALLET_DIR.to_string(),
                error: e,
            })?;
            return Ok(Self::default());
        }

        let wallet_file = format!("{}/{}", WALLET_DIR, WALLET_FILE);
        let file_path = Path::new(&wallet_file);
        if !file_path.exists() {
            return Ok(Self::default());
        }

        let mut file = File::open(file_path).map_err(|e| WalletError::StorageRead {
            path: wallet_file.clone(),
            error: e,
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| WalletError::StorageRead {
                path: wallet_file,
                error: e,
            })?;

        let wallets =
            serde_json::from_str(&contents).map_err(|e| WalletError::JsonParse { error: e })?;

        Ok(wallets)
    }

    /// Saves wallet data to local storage.
    ///
    /// Serializes the wallets collection to JSON and writes it to the wallet file.
    /// Creates the .wallets directory if it doesn't exist.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the wallets are saved successfully
    /// * `Err(WalletError)` - If an error occurs while writing wallet data
    pub fn save(&self) -> Result<()> {
        // Create wallet directory if it doesn't exist
        let wallet_path = Path::new(WALLET_DIR);
        if !wallet_path.exists() {
            fs::create_dir_all(wallet_path).map_err(|e| WalletError::StorageCreate {
                path: WALLET_DIR.to_string(),
                error: e,
            })?;
        }

        let wallet_file = format!("{}/{}", WALLET_DIR, WALLET_FILE);
        let mut file = File::create(&wallet_file).map_err(|e| WalletError::StorageWrite {
            path: wallet_file.clone(),
            error: e,
        })?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| WalletError::JsonSerialize { error: e })?;

        file.write_all(json.as_bytes())
            .map_err(|e| WalletError::StorageWrite {
                path: wallet_file.clone(),
                error: e,
            })?;

        Ok(())
    }

    /// Adds a new wallet to the collection and saves to disk.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to associate with the wallet
    /// * `keypair` - The key pair for the wallet
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the wallet is added and saved successfully
    /// * `Err(WalletError)` - If an error occurs while saving  
    pub fn add_wallet(&mut self, name: &str, keypair: KeyPair) -> Result<()> {
        self.wallets.insert(name.to_string(), keypair);
        self.save()?;
        Ok(())
    }

    /// Gets a wallet by name from the collection.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the wallet to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(&KeyPair)` - The wallet's key pair if found
    /// * `None` - If no wallet with the given name exists
    pub fn get_wallet(&self, name: &str) -> Option<&KeyPair> {
        self.wallets.get(name)
    }

    /// Resolves a wallet name or public key to an address.
    ///
    /// Attempts to resolve the input as:
    /// 1. A wallet name in the collection
    /// 2. A public key that matches one of the wallets in the collection
    /// 3. A valid public key in general
    ///
    /// # Arguments
    ///
    /// * `name_or_key` - Wallet name or public key to resolve
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The resolved public key address
    /// * `None` - If the input cannot be resolved to a valid address
    pub fn resolve_address(&self, name_or_key: &str) -> Option<String> {
        // Check if it's a wallet name we know
        if let Some(keypair) = self.wallets.get(name_or_key) {
            return Some(keypair.public_key.clone());
        }

        // Check if it's a valid public key stored with us
        if self.wallets.values().any(|kp| kp.public_key == name_or_key) {
            return Some(name_or_key.to_string());
        }

        // Check if it's a valid public key in general
        if let Ok(public_key_bytes) = hex::decode(name_or_key) {
            if PublicKey::from_slice(&public_key_bytes).is_ok() {
                return Some(name_or_key.to_string());
            }
        }

        None
    }
}
