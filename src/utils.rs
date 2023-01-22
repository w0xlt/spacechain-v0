use std::{fs::File, io::Read};

use bdk::{bitcoin::{Transaction, blockdata::{script, opcodes}, Script}, Wallet, database::SqliteDatabase, SyncOptions, blockchain::{RpcConfig, rpc::{Auth, RpcSyncParams}, RpcBlockchain, ConfigurableBlockchain}};
use bdk::blockchain::{ElectrumBlockchain, Blockchain};
use bdk::electrum_client::Client;

use crate::config_file::ConfigFile;

pub fn broadcast_tx(transaction: &Transaction)
{
    let electrum_url = "tcp://127.0.0.1:50001";

    let blockchain = ElectrumBlockchain::from(Client::new(electrum_url).unwrap());

    blockchain.broadcast(transaction).unwrap();
}

pub fn build_bump_script(add_op_3: bool) -> Script
{
    let mut builder = script::Builder::new();

    if add_op_3 {
        builder = builder.push_opcode(opcodes::all::OP_PUSHBYTES_3);
    }

    builder
        .push_opcode(opcodes::all::OP_PUSHBYTES_0)
        .push_opcode(opcodes::all::OP_CSV)
        .push_opcode(opcodes::all::OP_1ADD)
        .into_script()
}

pub fn sync_wallet_electrum(cfg: &ConfigFile, wallet: &Wallet<SqliteDatabase>)
{
    let electrum_url = &cfg.electrum_url;

    let blockchain = ElectrumBlockchain::from(Client::new(electrum_url).unwrap());

    wallet.sync(&blockchain, SyncOptions::default()).unwrap();
}

fn read_birthdate(wallet_name: &str) -> u64 {

    let mut birthdate:u64 = 0;

    let home_dir = dirs::home_dir();
    let mut path = home_dir.unwrap();
    path.push(".spacechains");
    path.push(wallet_name);
    path.push("birthdate");

    let key_file = File::open(path.clone()).ok();

    if let Some(mut file) = key_file {
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer).unwrap();

        let mut arr = [0; 8];
        arr.copy_from_slice(&buffer[0..buffer.len()]);
        birthdate = u64::from_ne_bytes(arr);
    }

    birthdate

}

pub fn sync_wallet_rpc(cfg: &ConfigFile, wallet_name: &str, wallet: &Wallet<SqliteDatabase>)
{
    let birthdate = read_birthdate(&wallet_name);

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

pub fn sync_wallet(cfg: &ConfigFile, wallet_name: &str, wallet: &Wallet<SqliteDatabase>) {
    match cfg.blockchain.as_str()  {
        "electrum" => {
            sync_wallet_electrum(cfg, wallet);
        },
        "bitcoin_rpc" => {
            sync_wallet_rpc(cfg, wallet_name, wallet);
        }
        _ => panic!("Unexpected blockchain."),
    }
}