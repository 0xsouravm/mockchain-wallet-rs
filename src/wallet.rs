use crate::errors::{Result, WalletError};
use crate::models::{KeyPair, Wallets};
use crate::proto::blockchain::{
    blockchain_service_client::BlockchainServiceClient,
    BalanceRequest,
    Block as ProtoBlock, // Added
    FaucetRequest,
    FaucetResponse as ProtoFaucetResponse, // Added Response type
    GetBlockRequest,                       // Added
    GetStateRequest,                       // Added
    HistoryRequest,                        // Added
    Transaction,                           // Renamed for clarity
};
use secp256k1::{Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::Request;

/// Client for interacting with the blockchain service.
///
/// Provides functionality for managing wallets and performing
/// blockchain operations like checking balances and sending transactions.
pub struct WalletClient {
    client: BlockchainServiceClient<tonic::transport::Channel>,
    wallets: Wallets,
}

impl WalletClient {
    /// Creates a new wallet client connected to the blockchain service.
    ///
    /// Establishes a connection to the blockchain service at the default address
    /// (http://[::1]:50051) and loads wallet data from local storage.
    ///
    /// # Returns
    ///
    /// * `Ok(WalletClient)` - A new client instance ready to use
    /// * `Err(WalletError)` - If connection to the service fails or wallet data cannot be loaded
    pub async fn new() -> Result<Self> {
        let client = BlockchainServiceClient::connect("http://[::1]:50051").await?;
        let wallets = Wallets::load()?;
        Ok(WalletClient { client, wallets })
    }

    /// Creates a new wallet with the given name.
    ///
    /// Generates a new secp256k1 key pair and stores it in local storage
    /// associated with the provided name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to assign to the new wallet
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the wallet is created successfully
    /// * `Err(WalletError::WalletExists)` - If a wallet with the given name already exists
    /// * `Err(WalletError)` - If an error occurs while generating or storing the wallet
    pub fn create_wallet(&mut self, name: &str) -> Result<()> {
        if self.wallets.get_wallet(name).is_some() {
            return Err(WalletError::WalletExists(name.to_string()));
        }

        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

        let secret_hex = hex::encode(secret_key.secret_bytes());
        let public_hex = hex::encode(public_key.serialize());

        let keypair = KeyPair {
            private_key: secret_hex,
            public_key: public_hex,
        };

        self.wallets.add_wallet(name, keypair)?;
        Ok(())
    }

    /// Gets the balance for a wallet.
    ///
    /// Queries the blockchain service for the current balance of the wallet
    /// specified by name or public key.
    ///
    /// # Arguments
    ///
    /// * `wallet_name_or_key` - Name of a wallet in local storage or a public key
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - The wallet's balance in coins
    /// * `Err(WalletError::WalletNotFound)` - If the wallet or address cannot be resolved
    /// * `Err(WalletError)` - If an error occurs while querying the blockchain
    pub async fn get_balance(&mut self, wallet_name_or_key: &str) -> Result<u64> {
        let address = self.wallets.resolve_address(wallet_name_or_key)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_name_or_key.to_string()))?;
        
        let request = Request::new(BalanceRequest {
            address
        });
        
        let response = self.client.get_balance(request).await?;
        Ok(response.into_inner().balance)
    }

    /// Sends a transaction from one wallet to another.
    ///
    /// Signs and submits a transaction to transfer coins from the sender's wallet
    /// to the recipient. The recipient can be specified by wallet name or public key.
    ///
    /// # Arguments
    ///
    /// * `from_wallet` - Name of the sender's wallet in local storage
    /// * `to_name_or_key` - Name or public key of the recipient
    /// * `amount` - Number of coins to transfer
    ///
    /// # Returns
    ///
    /// * `Ok(bool)` - True if the transaction was successful
    /// * `Err(WalletError::WalletNotFound)` - If the sender wallet cannot be found
    /// * `Err(WalletError::AddressInvalid)` - If the recipient address is invalid
    /// * `Err(WalletError)` - If an error occurs during signing or submission
    pub async fn send_transaction(
        &mut self,
        from_wallet: &str,
        to_name_or_key: &str,
        amount: u64,
    ) -> Result<bool> {
        // Get sender's keypair
        let keypair = self.wallets.get_wallet(from_wallet)
            .ok_or_else(|| WalletError::WalletNotFound(from_wallet.to_string()))?;

        // Resolve recipient
        let to_address = self.wallets.resolve_address(to_name_or_key)
            .ok_or_else(|| WalletError::AddressInvalid(to_name_or_key.to_string()))?;

        // Decode private key
        let secret_key_bytes = hex::decode(&keypair.private_key)?;
        let secret_key = SecretKey::from_slice(&secret_key_bytes)
            .map_err(|e| WalletError::InvalidPrivateKey { 
                message: e.to_string() 
            })?;
        
        // Get current timestamp for transaction
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| WalletError::SystemTimeError { 
                message: e.to_string() 
            })?
            .as_secs();

        // Create message to sign
        let mut hasher = Sha256::new();
        hasher.update(
            serde_json::to_string(&(
                &keypair.public_key,
                &to_address,
                amount,
                timestamp,
            )).map_err(|e| WalletError::JsonSerialize { 
                error: e 
            })?.as_bytes()
        );

        let message = hasher.finalize();
        
        // Sign transaction
        let secp = Secp256k1::new();
        let msg = secp256k1::Message::from_slice(&message)
            .map_err(|e| WalletError::SigningFailed { 
                message: e.to_string() 
            })?;
            
        let signature = secp.sign_ecdsa(&msg, &secret_key);
        
        // Create and send transaction
        let transaction = Transaction {
            from: keypair.public_key.clone(),
            to: to_address,
            amount,
            timestamp,
            signature: signature.serialize_compact().to_vec(),
        };
        
        let request = Request::new(transaction);
        let response = self.client.submit_transaction(request).await?;
        let response_inner = response.into_inner();
        if !response_inner.success {
            return Err(WalletError::TransactionFailed { 
                message: response_inner.message 
            });
        }
        
        Ok(response_inner.success)
    }

    /// Requests funds from the blockchain's faucet.
    ///
    /// Submits a request to the blockchain's faucet service to send funds to
    /// the specified wallet, typically used for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `wallet_name` - Name of the wallet to receive funds
    ///
    /// # Returns
    /// 
    /// * `Ok(u64)` - The amount of coins received
    /// * `Err(WalletError::WalletNotFound)` - If the wallet cannot be found
    /// * `Err(WalletError)` - If an error occurs with the blockchain service
    pub async fn request_faucet(&mut self, wallet_name: &str) -> Result<u64> {
        let keypair = self.wallets.get_wallet(wallet_name)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_name.to_string()))?;

        let request = Request::new(FaucetRequest {
            address: keypair.public_key.clone(),
        });
        
        let response = self.client.request_faucet(request).await?;
        let response_inner = response.into_inner();
        
        if !response_inner.success {
            return Err(WalletError::FaucetFailed { 
                message: response_inner.message 
            });
        }
        
        Ok(response_inner.amount)
    }

    /// Lists all wallets in local storage.
    /// 
    /// # Returns
    /// 
    /// A vector of (name, keypair) tuples for all wallets in local storage.
    pub fn list_wallets(&self) -> Vec<(String, KeyPair)> {
        self.wallets.wallets.iter()
            .map(|(name, keypair)| (name.clone(), keypair.clone()))
            .collect()
    }

    /// Gets a wallet by name from local storage.
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
        self.wallets.get_wallet(name)
    }
}
