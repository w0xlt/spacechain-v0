use bdk::{Wallet, database::SqliteDatabase, wallet::{AddressIndex, export::FullyNodedExport}};
use config_file::ConfigFile;
use serde_json::{json, Value};

mod keystore;
mod wallet_manager;
mod covenant;
mod utils;
mod config_file;

fn main() {
    // env_logger::init();
    run();
}

fn run() {
    let (cfg, cfg_path) = config_file::create_or_get_default();

    let args: Vec<String> = std::env::args().collect();

    let error_message = "Please run this with the following methods: new_address balance backup broadcast config_file";
    if args.len() == 1 {
        println!("{}", error_message);
        return;
    };
    let method = args[1].as_str();
    if method != "new_address" && method != "backup" && method != "broadcast" && method != "balance" && method != "config_file" {
        println!("{}", error_message);
        return;
    }

    match method {
        "new_address" => {
            if args.len() > 2 {
                println!("new_address method does not require argument.");
                return;
            }
            new_address(&cfg);
        },
        "backup" => {
            if args.len() > 2 {
                println!("backup method does not require argument.");
                return;
            }
            backup(&cfg);
        },
        "balance" => {
            if args.len() > 2 {
                println!("balance method does not require argument.");
                return;
            }
            balance(&cfg);
        },
        "config_file" => {
            if args.len() > 2 {
                println!("config_file method does not require argument.");
                return;
            }
            config_file(&cfg, &cfg_path);
        },
        "broadcast" => {
            if args.len() != 4 {
                println!("broadcast method requires the fee amount and the data message (e.g. broadcast 10000 \"hello world\").");
                return;
            }
            let fee_amount = args[2].parse::<u64>().unwrap();
            let data_message = args[3].as_str();

            if data_message.len() > 80 {
                println!("Error: The data_message must be up to 80 bytes.");
            }

            broadcast(&cfg, data_message, fee_amount);
        },
        _ => println!("{}", error_message),
    }
}

fn config_file(cfg: &ConfigFile, path: &String)
{
    println!("Config file located in {}", path);

    let obj = json!(cfg);

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}

fn get_user_wallet(cfg: &ConfigFile) -> Wallet<SqliteDatabase>
{
    let wallet_name = "default";

    let network = cfg.get_network().unwrap();

    let xprv_from_restore = keystore::get_wallet_xpriv(wallet_name, network);

    if xprv_from_restore == None {
        panic!("Unable to retrieve the wallet key!");
    }

    let xprv = xprv_from_restore.unwrap();

    let wallet = wallet_manager::load_wallet(wallet_name, &xprv, network);

    // utils::sync_wallet(&wallet);
    // utils::sync_wallet_rpc(String::from_str("default").unwrap(), &wallet);
    utils::sync_wallet(&cfg, "default", &wallet);

    wallet
}

fn new_address(cfg: &ConfigFile) {
    let wallet = get_user_wallet(cfg);

    let addr = wallet.get_address(AddressIndex::New).unwrap();

    let obj = json!({"address": addr.address.to_string(), "index": addr.index});

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}

fn backup(cfg: &ConfigFile) {
    let wallet = get_user_wallet(cfg);

    let export = FullyNodedExport::export_wallet(&wallet, "exported wallet", true)
        .map_err(ToString::to_string)
        .map_err(bdk::Error::Generic)
        .unwrap();

    let value: Value = serde_json::from_str(export.to_string().as_str()).unwrap();

    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}

fn balance(cfg: &ConfigFile) {
    let wallet = get_user_wallet(cfg);

    let balance = wallet.get_balance().unwrap();

    let obj = json!({"immature": balance.immature, "trusted_pending": balance.trusted_pending,
        "untrusted_pending": balance.untrusted_pending, "confirmed": balance.confirmed});

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());
}

fn broadcast(cfg: &ConfigFile, data_message: &str, fee_amount: u64) {

    let wallet = get_user_wallet(cfg);

    let balance = wallet.get_balance().unwrap().confirmed;
    if balance < fee_amount {
        println!("Insufficient amount of {} sats in wallet to pay a fee of {} sats.", balance, fee_amount);
        return;
    }

    let (covenant_transaction,satisfaction_weight) = covenant::create_convenant_transaction(cfg);

    let cpfp_transaction = wallet_manager::create_cpfp_transaction(cfg, &wallet, data_message, &covenant_transaction, satisfaction_weight, fee_amount);

    utils::broadcast_tx(&covenant_transaction);
    utils::broadcast_tx(&cpfp_transaction);

    let obj = json!({"covenant_transaction_id": covenant_transaction.txid(), "cpfp_transaction_id": cpfp_transaction.txid()});

    println!("{}", serde_json::to_string_pretty(&obj).unwrap());

    // use bdk::bitcoin::consensus::serialize;

    // let consensus_convenant_encoded = serialize(&covenant_transaction);
    // println!("Covenant tx: {:02x?}", consensus_convenant_encoded);

    // let consensus_cpfp_encoded = serialize(&cpfp_transaction);
    // println!("cpfp tx: {:02x?}", consensus_cpfp_encoded);
}