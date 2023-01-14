use std::path::PathBuf;

use bdk::blockchain::{Blockchain, ElectrumBlockchain};
use bdk::electrum_client::Client;
use bdk::{
    bitcoin::{
        blockdata::{opcodes, script},
        secp256k1::Secp256k1,
        Network, Script, Transaction,
    },
    blockchain::{
        rpc::{Auth, RpcSyncParams},
        ConfigurableBlockchain, RpcBlockchain, RpcConfig,
    },
    database::SqliteDatabase,
    wallet::wallet_name_from_descriptor,
    Error, SyncOptions, Wallet,
};

use crate::config_file::ConfigFile;

pub fn broadcast_tx(cfg: &ConfigFile, transaction: &Transaction) -> Result<(), Error> {
    let electrum_url = &cfg.electrum_url;

    let blockchain = ElectrumBlockchain::from(Client::new(electrum_url).unwrap());

    blockchain.broadcast(transaction)
}

pub fn build_bump_script() -> Script {
    script::Builder::new()
        .push_opcode(opcodes::all::OP_PUSHBYTES_0)
        .push_opcode(opcodes::all::OP_CSV)
        .push_opcode(opcodes::all::OP_1ADD)
        .into_script()
}

pub fn sync_wallet_electrum(cfg: &ConfigFile, wallet: &Wallet<SqliteDatabase>) {
    let electrum_url = &cfg.electrum_url;

    let blockchain = ElectrumBlockchain::from(Client::new(electrum_url).unwrap());

    wallet.sync(&blockchain, SyncOptions::default()).unwrap();
}

pub fn sync_wallet_rpc(
    cfg: &ConfigFile,
    wallet_name: &str,
    wallet: &Wallet<SqliteDatabase>,
    birthdate: u64,
) {
    let sync_params = RpcSyncParams {
        start_time: birthdate,
        ..Default::default()
    };

    let config = RpcConfig {
        url: cfg.bitcoind_url.to_string(),
        auth: Auth::Cookie {
            file: cfg.bitcoind_auth_file.to_string().into(),
        },
        network: cfg.get_network().unwrap(),
        wallet_name: wallet_name.to_string(),
        sync_params: Some(sync_params),
    };

    let blockchain = RpcBlockchain::from_config(&config).unwrap();

    wallet.sync(&blockchain, SyncOptions::default()).unwrap();
}

pub fn sync_wallet(
    cfg: &ConfigFile,
    wallet_name: &str,
    wallet: &Wallet<SqliteDatabase>,
    birthdate: Option<u64>,
) {
    match cfg.blockchain.as_str() {
        "electrum" => {
            sync_wallet_electrum(cfg, wallet);
        }
        "bitcoin_rpc" => {
            sync_wallet_rpc(cfg, wallet_name, wallet, birthdate.unwrap_or(0));
        }
        _ => panic!("Unexpected blockchain."),
    }
}

pub fn get_keystore_db_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap();

    path.push(".spacechains");

    std::fs::create_dir_all(path.clone()).unwrap();

    path.push("wallet.db");

    path
}

pub fn get_bdk_wallet_path(
    external_descriptor: &String,
    internal_descriptor: &Option<String>,
    network: Network,
) -> PathBuf {
    let wallet_name = wallet_name_from_descriptor(
        external_descriptor,
        internal_descriptor.as_ref(),
        network,
        &Secp256k1::new(),
    )
    .unwrap();

    let mut path = dirs::home_dir().unwrap();

    path.push(".spacechains");
    path.push(wallet_name);

    std::fs::create_dir_all(path.clone()).unwrap();

    path.push("database");

    path
}
