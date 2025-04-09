use structopt::StructOpt;

/// Commands supported by the blockchain wallet CLI.
/// 
/// Defines the command-line interface structure using StructOpt.
#[derive(StructOpt)]
pub enum Command {
    /// Creates a new wallet with the specified name
    #[structopt(name = "new")]
    CreateWallet {
        /// Name to assign to the new wallet
        #[structopt(name = "name")]
        name: String,
    },
    
    /// Lists all wallets in local storage
    #[structopt(name = "list")]
    ListWallets,
    
    /// Gets the balance for a wallet
    #[structopt(name = "balance")]
    GetBalance {
        /// Name of the wallet to check
        #[structopt(name = "wallet")]
        wallet_name: String,
    },
    
    /// Sends a transaction from one wallet to another
    #[structopt(name = "send")]
    SendTransaction {
        /// Name of the sender's wallet
        #[structopt(name = "from")]
        from_wallet: String,
        
        /// Name or address of the recipient
        #[structopt(name = "to")]
        to_wallet: String,
        
        /// Amount of coins to send
        #[structopt(name = "amount")]
        amount: u64,
    },
    
    /// Requests funds from the blockchain faucet
    #[structopt(name = "faucet")]
    RequestFaucet {
        /// Name of the wallet to receive funds
        #[structopt(name = "wallet")]
        wallet_name: String,
    },

    /// Get the transaction history for a wallet (by name or address)
    GetHistory {
        /// Wallet name or public key address (hex)
        wallet_name_or_key: String,
    },

    /// Get the entire state of the blockchain (all blocks)
    GetState,

    /// Get a specific block by its index
    GetBlock {
        /// Index of the block to retrieve
        index: u64,
    },
}
