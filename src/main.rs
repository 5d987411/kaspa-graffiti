use kaspa_graffiti::commands::{generate_wallet, load_wallet, get_balance, get_utxos, send_graffiti, generate_hd_wallet, load_hd_wallet, derive_address_from_seed, derive_many_addresses};
use kaspa_graffiti::rpc::PUBLIC_TESTNET10_RPC;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }
    
    // Check for --rpc flag
    let mut rpc_url: Option<&str> = None;
    let mut cmd_args: Vec<&str> = vec![];
    
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--rpc" && i + 1 < args.len() {
            rpc_url = Some(&args[i + 1]);
            i += 2;
        } else {
            cmd_args.push(&args[i]);
            i += 1;
        }
    }
    
    if cmd_args.is_empty() {
        print_usage();
        return;
    }
    
    let cmd = cmd_args[0];
    
    match cmd {
        "generate" => {
            match generate_wallet().await {
                Ok(info) => {
                    println!("{{");
                    println!("  \"private_key\": \"{}\",", info.private_key);
                    println!("  \"public_key\": \"{}\",", info.public_key);
                    println!("  \"address\": \"{}\",", info.address);
                    println!("  \"network\": \"{}\"", info.network);
                    println!("}}");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "load" => {
            if cmd_args.len() < 2 {
                eprintln!("Usage: kaspa-graffiti-cli load <private_key>");
                return;
            }
            match load_wallet(&cmd_args[1]).await {
                Ok(info) => {
                    println!("{{");
                    println!("  \"private_key\": \"{}\",", info.private_key);
                    println!("  \"public_key\": \"{}\",", info.public_key);
                    println!("  \"address\": \"{}\",", info.address);
                    println!("  \"network\": \"{}\"", info.network);
                    println!("}}");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "balance" => {
            if cmd_args.len() < 2 {
                eprintln!("Usage: kaspa-graffiti-cli balance <address> [--rpc <url>]");
                return;
            }
            let rpc = rpc_url.or(Some(PUBLIC_TESTNET10_RPC));
            match get_balance(&cmd_args[1], rpc).await {
                Ok(info) => {
                    println!("{{");
                    println!("  \"address\": \"{}\",", info.address);
                    println!("  \"balance\": {},", info.balance);
                    println!("  \"kas\": {:.8}", info.balance as f64 / 100_000_000.0);
                    println!("}}");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "utxos" => {
            if cmd_args.len() < 2 {
                eprintln!("Usage: kaspa-graffiti-cli utxos <address> [--rpc <url>]");
                return;
            }
            let rpc = rpc_url.or(Some(PUBLIC_TESTNET10_RPC));
            match get_utxos(&cmd_args[1], rpc).await {
                Ok(utxos) => {
                    println!("[");
                    for (i, utxo) in utxos.iter().enumerate() {
                        println!("  {{");
                        println!("    \"txid\": \"{}\",", utxo.txid);
                        println!("    \"vout\": {},", utxo.vout);
                        println!("    \"amount\": {},", utxo.amount);
                        println!("    \"kas\": {:.8}", utxo.amount as f64 / 100_000_000.0);
                        println!("    \"script_pubkey\": \"{}\"", utxo.script_pubkey);
                        if i < utxos.len() - 1 {
                            println!("  }},");
                        } else {
                            println!("  }}");
                        }
                    }
                    println!("]");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "graffiti" => {
            if cmd_args.len() < 3 {
                eprintln!("Usage: kaspa-graffiti-cli graffiti <private_key> <message> [mimetype] [fee_rate] [--rpc <url>]");
                return;
            }
            let private_key = &cmd_args[1];
            let message = &cmd_args[2];
            let mimetype = cmd_args.get(3).map(|s| *s);
            let fee_rate = cmd_args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1000u64);
            let rpc = rpc_url.or(Some(PUBLIC_TESTNET10_RPC));
            
            println!("Sending graffiti message...");
            println!("Message: {}", message);
            println!("Fee rate: {} sompi", fee_rate);
            
            match send_graffiti(private_key, message, mimetype, rpc, fee_rate).await {
                Ok(result) => {
                    println!("\n✓ Transaction sent successfully!");
                    println!("{{");
                    println!("  \"txid\": \"{}\",", result.txid);
                    println!("  \"fee\": {},", result.fee);
                    println!("  \"change\": {},", result.change);
                    println!("  \"address\": \"{}\"", result.address);
                    println!("}}");
                }
                Err(e) => {
                    eprintln!("\n✗ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "hd-generate" => {
            match generate_hd_wallet().await {
                Ok(info) => {
                    println!("{{");
                    println!("  \"seed\": \"{}\",", info.seed);
                    println!("  \"address\": \"{}\",", info.address);
                    println!("  \"network\": \"{}\"", info.network);
                    println!("}}");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "hd-load" => {
            if cmd_args.len() < 2 {
                eprintln!("Usage: kaspa-graffiti-cli hd-load <seed>");
                return;
            }
            match load_hd_wallet(&cmd_args[1]).await {
                Ok(info) => {
                    println!("{{");
                    println!("  \"seed\": \"{}\",", info.seed);
                    println!("  \"address\": \"{}\",", info.address);
                    println!("  \"network\": \"{}\"", info.network);
                    println!("}}");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "derive-address" => {
            if cmd_args.len() < 3 {
                eprintln!("Usage: kaspa-graffiti-cli derive-address <seed> <index> [change]");
                return;
            }
            let seed = cmd_args[1];
            let index: u32 = cmd_args[2].parse().unwrap_or(0);
            let is_change = cmd_args.get(3).map(|s| *s == "change" || *s == "true").unwrap_or(false);
            
            match derive_address_from_seed(seed, index, is_change).await {
                Ok(info) => {
                    println!("{{");
                    println!("  \"index\": {},", info.index);
                    println!("  \"address\": \"{}\",", info.address);
                    println!("  \"private_key\": \"{}\",", info.private_key);
                    println!("  \"public_key\": \"{}\",", info.public_key);
                    println!("  \"is_change\": {}", info.is_change);
                    println!("}}");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        "derive-many" => {
            if cmd_args.len() < 3 {
                eprintln!("Usage: kaspa-graffiti-cli derive-many <private_key> <count>");
                return;
            }
            let private_key = cmd_args[1];
            let count: u32 = cmd_args[2].parse().unwrap_or(1);
            
            match derive_many_addresses(private_key, count, false).await {
                Ok(addresses) => {
                    println!("[");
                    for (i, addr) in addresses.iter().enumerate() {
                        println!("  {{");
                        println!("    \"index\": {},", addr.index);
                        println!("    \"address\": \"{}\",", addr.address);
                        println!("    \"private_key\": \"{}\",", addr.private_key);
                        println!("    \"public_key\": \"{}\",", addr.public_key);
                        println!("    \"is_change\": {}", addr.is_change);
                        println!("  }}{}", if i < addresses.len() - 1 { "," } else { "" });
                    }
                    println!("]");
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Kaspa Graffiti CLI");
    println!();
    println!("Usage:");
    println!("  kaspa-graffiti-cli generate                      Generate a new wallet");
    println!("  kaspa-graffiti-cli load <key>                    Load wallet from private key");
    println!("  kaspa-graffiti-cli balance <address> [--rpc <url>]  Get address balance");
    println!("  kaspa-graffiti-cli utxos <address> [--rpc <url>]    Get address UTXOs");
    println!("  kaspa-graffiti-cli graffiti <key> <msg> [mime] [fee] [--rpc <url>]  Send graffiti");
    println!();
    println!("HD Wallet Commands:");
    println!("  kaspa-graffiti-cli hd-generate                   Generate a new HD wallet");
    println!("  kaspa-graffiti-cli hd-load <seed>                Load HD wallet from seed");
    println!("  kaspa-graffiti-cli derive-address <seed> <index> [change]  Derive address from seed");
    println!("  kaspa-graffiti-cli derive-many <key> <count>     Derive multiple addresses");
    println!();
    println!("Options:");
    println!("  --rpc <url>    RPC endpoint (default: {})", PUBLIC_TESTNET10_RPC);
    println!();
    println!("Examples:");
    println!("  kaspa-graffiti-cli generate");
    println!("  kaspa-graffiti-cli hd-generate");
    println!("  kaspa-graffiti-cli derive-address <seed> 0");
    println!("  kaspa-graffiti-cli derive-many <private_key> 5");
    println!("  kaspa-graffiti-cli balance kaspatest:qq...");
    println!("  kaspa-graffiti-cli graffiti <private_key> \"Hello Kaspa!\" text/plain 1000");
}
