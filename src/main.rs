use bdk::{
    bitcoin::{secp256k1::Secp256k1, Network},
    database::SqliteDatabase,
    wallet::{export::FullyNodedExport, wallet_name_from_descriptor, AddressIndex},
    Wallet,
};
use clap::{command, Parser, Subcommand};
use config_file::ConfigFile;
use serde_json::{json, Value};

mod config_file;
mod covenant;
mod keystore;
mod utils;
mod wallet_manager;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet, with a new random BIP 84 extended key
    CreateWallet { wallet_name: String },
    /// Import a wallet, given external and internal descriptors
    ImportWallet {
        wallet_name: String,
        external_descriptor: String,
        internal_descriptor: Option<String>,
    },
    /// Get a wallet balance
    GetBalance { wallet_name: String },
    /// Get a new address
    GetNewAddress { wallet_name: String },
    /// Back up your wallet
    Backup { wallet_name: String },
    /// Show configuration file
    ConfigFile,
    /// Mine a new spacechain block
    Mine {
        wallet_name: String,
        data_message: String,
        fee_amount: u64,
    },
}

fn main() {
    let cli = Cli::parse();

    let (cfg, cfg_path) = config_file::create_or_get_default();

    let network = cfg.get_network().unwrap();

    match &cli.command {
        Commands::CreateWallet { wallet_name } => {
            create_wallet(wallet_name, network);
        }
        Commands::ImportWallet {
            wallet_name,
            external_descriptor,
            internal_descriptor,
        } => {
            import_wallet(wallet_name, external_descriptor, internal_descriptor);
        }
        Commands::GetBalance { wallet_name } => {
            get_balance(&cfg, wallet_name);
        }
        Commands::GetNewAddress { wallet_name } => {
            get_new_address(&cfg, wallet_name);
        }
        Commands::Backup { wallet_name } => {
            backup2(&cfg, wallet_name);
        }
        Commands::ConfigFile => {
            config_file(&cfg, &cfg_path);
        }
        Commands::Mine {
            wallet_name,
            data_message,
            fee_amount,
        } => {
            mine(&cfg, wallet_name, data_message, *fee_amount);
        }
    }
}

fn get_user_wallet(cfg: &ConfigFile, wallet_name: &String) -> Wallet<SqliteDatabase> {
    let path = utils::get_keystore_db_path();

    let wallet_data = keystore::load_descriptors(&path, wallet_name);

    if wallet_data == None {
        panic!("Wallet {wallet_name} not found !")
    }

    let wallet_data = wallet_data.unwrap();

    let external_descriptor = wallet_data.0;
    let internal_descriptor = wallet_data.1;
    let birthdate = wallet_data.2;

    let network = cfg.get_network().unwrap();

    let wallet_name = wallet_name_from_descriptor(
        &external_descriptor,
        internal_descriptor.as_ref(),
        network,
        &Secp256k1::new(),
    )
    .unwrap();

    let wallet = wallet_manager::load_wallet(&external_descriptor, &internal_descriptor, network);

    utils::sync_wallet(&cfg, wallet_name.as_str(), &wallet, Some(birthdate));

    wallet
}

fn config_file(cfg: &ConfigFile, path: &String) {
    println!("Config file located in {}", path);

    let obj = json!(cfg);

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}

fn backup2(cfg: &ConfigFile, wallet_name: &String) {
    let wallet = get_user_wallet(cfg, wallet_name);

    let export = FullyNodedExport::export_wallet(&wallet, wallet_name, true)
        .map_err(ToString::to_string)
        .map_err(bdk::Error::Generic)
        .unwrap();

    let value: Value = serde_json::from_str(export.to_string().as_str()).unwrap();

    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}

fn get_new_address(cfg: &ConfigFile, wallet_name: &String) {
    let wallet = get_user_wallet(cfg, wallet_name);

    let addr = wallet.get_address(AddressIndex::New).unwrap();

    let obj = json!({"address": addr.address.to_string(), "index": addr.index});

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}

fn create_wallet(wallet_name: &String, network: Network) {
    keystore::create_new_wallet_desc(wallet_name, network);
    println!("Wallet created successfully !");
}

fn import_wallet(
    wallet_name: &String,
    external_descriptor: &String,
    internal_descriptor: &Option<String>,
) {
    keystore::import_wallet_desc(wallet_name, external_descriptor, internal_descriptor);
    println!("Wallet imported successfully !");
}

fn get_balance(cfg: &ConfigFile, wallet_name: &String) {
    let wallet = get_user_wallet(cfg, wallet_name);

    let balance = wallet.get_balance().unwrap();

    let obj = json!({"immature": balance.immature, "trusted_pending": balance.trusted_pending,
        "untrusted_pending": balance.untrusted_pending, "confirmed": balance.confirmed});

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}

fn mine(cfg: &ConfigFile, wallet_name: &String, data_message: &str, fee_amount: u64) {
    let covenant_wallet = covenant::load_convenant_wallet_from_db(&cfg);
    utils::sync_wallet(&cfg, "covenant", &covenant_wallet, None);

    let covenant_result = covenant::get_covenant_tx_from_db(&covenant_wallet);

    if covenant_result == None {
        println!("No covenant transaction found.");
        return;
    }

    let (previous_covenant_txid, covenant_transaction, satisfaction_weight) =
        covenant_result.unwrap();

    let cpfp_wallet = get_user_wallet(cfg, wallet_name);

    let cpfp_transaction = wallet_manager::create_cpfp_transaction(
        &cpfp_wallet,
        data_message,
        &covenant_transaction,
        satisfaction_weight,
        fee_amount,
    );

    // let tx_bytes = serialize(&cpfp_transaction);
    // let hex_tx = tx_bytes.to_hex();
    // println!("cpfp_transaction: {}", hex_tx);

    let covenant_result = utils::broadcast_tx(&cfg, &covenant_transaction);
    match covenant_result {
        Ok(_) => {},
        Err(err) => {
            if err.to_string() == "Electrum(Protocol(String(\"sendrawtransaction RPC error: {\\\"code\\\":-26,\\\"message\\\":\\\"non-BIP68-final\\\"}\")))" {
                println!("The previous covenant transaction {} has not yet been confirmed, causing non-BIP68-final validation error. Please wait for at least one confirmation.", previous_covenant_txid);
                return;
            } else {
                panic!("{}", err);
            }
        },
    }

    loop {
        let cpfp_result = utils::broadcast_tx(&cfg, &cpfp_transaction);
        match cpfp_result {
            Ok(_) => { break; },
            Err(err) => {
                if err.to_string() != "Electrum(Protocol(String(\"sendrawtransaction RPC error: {\\\"code\\\":-25,\\\"message\\\":\\\"bad-txns-inputs-missingorspent\\\"}\")))" {
                    panic!("{}", err);
                }
            },
        }
    }

    let obj = json!({"covenant_transaction_id": covenant_transaction.txid(), "cpfp_transaction_id": cpfp_transaction.txid()});

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}
