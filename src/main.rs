mod commands;
mod errors;
mod models; // Assuming this exists for KeyPair
mod proto;
mod storage; // Assuming this exists for Wallets struct
mod wallet;

use chrono::{DateTime, Utc}; // For formatting block timestamp
use commands::Command;
use errors::WalletError;
use structopt::StructOpt;
use wallet::WalletClient;

/// Entry point for the blockchain wallet CLI application.
///
/// Parses command-line arguments and delegates to the `run` function.
/// Handles any errors that occur during execution and provides appropriate
/// error messages to the user.
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// The main application logic for the blockchain wallet CLI.
///
/// Processes the command-line arguments, initializes the wallet client,
/// and executes the requested command.
///
/// # Returns
///
/// * `Ok(())` - If the command executes successfully
/// * `Err(WalletError)` - If an error occurs during execution
async fn run() -> Result<(), WalletError> {
    let command = Command::from_args();
    let mut wallet = WalletClient::new().await?;

    match command {
        Command::CreateWallet { name } => match wallet.create_wallet(&name) {
            Ok(_) => {
                let keypair = wallet.get_wallet(&name).unwrap();
                println!("New wallet '{}' created!", name);
                println!("Address: {}", keypair.public_key);
            }
            Err(e) => {
                eprintln!("Error creating wallet: {}", e);
            }
        },

        Command::ListWallets => {
            let wallets = wallet.list_wallets();
            if wallets.is_empty() {
                println!("No wallets found. Create one with 'create-wallet --name <NAME>'");
            } else {
                println!("Your wallets:");
                for (name, keypair) in wallets {
                    println!(
                        "- {}: {}", // Simplified output
                        name, keypair.public_key
                    );
                }
            }
        }

        Command::GetBalance { wallet_name } => match wallet.get_balance(&wallet_name).await {
            Ok(balance) => println!("Balance for '{}': {} coins", wallet_name, balance),
            Err(e) => eprintln!("Error: {}", e),
        },

        Command::SendTransaction {
            from_wallet,
            to_wallet,
            amount,
        } => {
            match wallet
                .send_transaction(&from_wallet, &to_wallet, amount)
                .await
            {
                Ok(_) => println!("Transaction sent successfully!"),
                Err(e) => eprintln!("Error sending transaction: {}", e),
            }
        }

        Command::RequestFaucet { wallet_name } => match wallet.request_faucet(&wallet_name).await {
            Ok(amount) => println!("Received {} coins to wallet '{}'", amount, wallet_name),
            Err(e) => eprintln!("Error requesting from faucet: {}", e),
        },

        // --- New Commands ---
        Command::GetHistory { wallet_name_or_key } => {
            match wallet.get_history(&wallet_name_or_key).await {
                Ok(transactions) => {
                    if transactions.is_empty() {
                        println!("No transaction history found for '{}'.", wallet_name_or_key);
                    } else {
                        println!("Transaction History for '{}':", wallet_name_or_key);
                        for tx in transactions {
                            let dt = DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0)
                                .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                .unwrap_or_else(|| "Invalid Timestamp".to_string());
                            println!(
                                "- Time: {}, From: {}, To: {}, Amount: {}, Sig: {}...",
                                dt,
                                tx.from,
                                tx.to,
                                tx.amount,
                                tx.signature
                                    .iter()
                                    .take(8)
                                    .map(|b| format!("{:02x}", b))
                                    .collect::<String>()
                            );
                        }
                    }
                }
                Err(e) => eprintln!("Error getting history: {}", e),
            }
        }

        Command::GetState => {
            match wallet.get_state().await {
                Ok(blocks) => {
                    println!("Current Blockchain State ({} blocks):", blocks.len());
                    for block in blocks {
                        let dt = DateTime::<Utc>::from_timestamp(block.timestamp, 0)
                            .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| "Invalid Timestamp".to_string());
                        println!("--- Block {} ---", block.index);
                        println!("  Hash: {}", block.hash);
                        println!("  Prev Hash: {}", block.previous_hash);
                        println!("  Timestamp: {}", dt);
                        println!("  Nonce: {}", block.nonce);
                        println!("  Miner: {}", block.miner);
                        println!("  Transactions ({}):", block.transactions.len());
                        // Optionally print brief transaction info here too
                        // for tx in block.transactions {
                        //     println!("    - {} -> {} ({})", tx.from, tx.to, tx.amount);
                        // }
                        println!("---------------");
                    }
                }
                Err(e) => eprintln!("Error getting state: {}", e),
            }
        }

        Command::GetBlock { index } => {
            match wallet.get_block(index).await {
                Ok(Some(block)) => {
                    let dt = DateTime::<Utc>::from_timestamp(block.timestamp, 0)
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "Invalid Timestamp".to_string());
                    println!("--- Block {} ---", block.index);
                    println!("  Hash: {}", block.hash);
                    println!("  Prev Hash: {}", block.previous_hash);
                    println!("  Timestamp: {}", dt);
                    println!("  Nonce: {}", block.nonce);
                    println!("  Miner: {}", block.miner);
                    println!("  Transactions ({}):", block.transactions.len());
                    for tx in block.transactions {
                        let tx_dt = DateTime::<Utc>::from_timestamp(tx.timestamp as i64, 0)
                            .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                            .unwrap_or_else(|| "Invalid Timestamp".to_string());
                        println!(
                            "    - Time: {}, From: {}, To: {}, Amount: {}, Sig: {}...",
                            tx_dt,
                            tx.from,
                            tx.to,
                            tx.amount,
                            tx.signature
                                .iter()
                                .take(8)
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>()
                        );
                    }
                    println!("---------------");
                }
                Ok(None) => {
                    // Block not found is not an error state here
                    println!("Block with index {} not found.", index);
                }
                Err(e) => eprintln!("Error getting block {}: {}", index, e),
            }
        }
    }

    Ok(())
}
