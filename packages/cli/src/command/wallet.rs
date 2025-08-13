use layer_climb_cli::command::{create_wallet, WalletLog};

use crate::context::CliContext;

pub async fn handle_wallet_generate_env(ctx: &mut CliContext) {
    let keys = ["TEST_MNEMONIC", "CLI_MNEMONIC", "WAVS_SUBMISSION_MNEMONIC"];

    println!("Copy/paste this into your .env:\n");

    for key in keys {
        let (addr, mnemonic) = create_wallet(ctx.chain_config().unwrap(), &mut ctx.rng)
            .await
            .unwrap();
        println!("# Address: {}", addr);
        println!("{}=\"{}\"\n", key, mnemonic);
    }
}

pub async fn handle_wallet_generate_single(ctx: &mut CliContext) {
    let (addr, mnemonic) = create_wallet(ctx.chain_config().unwrap(), &mut ctx.rng)
        .await
        .unwrap();
    handle_wallet_log(WalletLog::Create { addr, mnemonic });
}

pub fn handle_wallet_log(log: WalletLog) {
    match log {
        WalletLog::Create { addr, mnemonic } => {
            println!("Wallet created!\n\n");
            println!("Address: {}", addr);
            println!("Mnemonic: {}", mnemonic);
        }
        WalletLog::Show { addr, balances } => {
            println!("Wallet address: {}", addr);
            println!("Balances:");
            for coin in balances {
                println!(" - {}: {}", coin.denom, coin.amount);
            }
        }
        WalletLog::Balance { addr, balance } => {
            println!(
                "Balance for {} - {}: {}",
                addr, balance.denom, balance.amount
            );
        }
        WalletLog::AllBalances { addr, balances } => {
            println!("All balances for {}:", addr);
            for coin in balances {
                println!(" - {}: {}", coin.denom, coin.amount);
            }
        }
        WalletLog::Transfer {
            to,
            amount,
            denom,
            tx_resp,
        } => {
            println!("Transfer successful!");
            println!("To: {}", to);
            println!("Amount: {} {}", amount, denom);
            println!("Transaction hash: {}", tx_resp.txhash);
        }
    }
}
