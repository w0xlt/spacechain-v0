use bdk::{
    bitcoin::{
        consensus::{deserialize, serialize},
        secp256k1::Secp256k1,
        Transaction, Txid,
    },
    database::{any::SqliteDbConfiguration, ConfigurableDatabase, SqliteDatabase},
    wallet::wallet_name_from_descriptor,
    KeychainKind, Wallet,
};
use rusqlite::{params, Connection};

use crate::config_file::ConfigFile;

fn load_convenant_wallet(cfg: &ConfigFile, public_descriptor: &String) -> Wallet<SqliteDatabase> {
    let home_dir = dirs::home_dir();

    let mut path = home_dir.unwrap();

    let network = cfg.get_network().unwrap();

    let wallet_name =
        wallet_name_from_descriptor(public_descriptor.as_str(), None, network, &Secp256k1::new())
            .unwrap();

    path.push(".spacechains");
    path.push(wallet_name);

    std::fs::create_dir_all(path.clone()).unwrap();

    path.push("database");

    let sqlite_db_configuration = SqliteDbConfiguration {
        path: path.into_os_string().into_string().unwrap(),
    };

    let sqlite_database =
        bdk::database::SqliteDatabase::from_config(&sqlite_db_configuration).unwrap();

    Wallet::new(public_descriptor.as_str(), None, network, sqlite_database).unwrap()
}

pub fn load_convenant_wallet_from_db(cfg: &ConfigFile) -> Wallet<SqliteDatabase> {
    let conn = Connection::open("convenant.db").unwrap();

    let mut stmt = conn
        .prepare("SELECT public_descriptor FROM convenant_descriptor")
        .unwrap();

    let convenant_iter = stmt
        .query_map([], |row| {
            let public_desc: String = row.get(0).unwrap();
            Ok(public_desc)
        })
        .unwrap();

    let mut descriptors: Vec<String> = Vec::new();

    for row in convenant_iter {
        let result = row.unwrap();
        descriptors.push(result);
    }

    assert!(descriptors.len() == 1);

    load_convenant_wallet(cfg, descriptors.get(0).unwrap())
}

pub fn get_covenant_tx_from_db(
    wallet: &Wallet<SqliteDatabase>,
) -> Option<(Txid, Transaction, usize)> {
    let conn = Connection::open("convenant.db").unwrap();

    for utxo in wallet.list_unspent().unwrap().iter() {
        let prev_txid_bytes = serialize(&utxo.outpoint.txid);

        let mut stmt = conn
            .prepare("SELECT previous_tx_id, tx_hex FROM convenant_txs WHERE previous_tx_id=(?1)")
            .unwrap();

        let convenant_iter = stmt
            .query_map(params![prev_txid_bytes], |row| {
                let previous_tx_id_bytes: Vec<u8> = row.get(0).unwrap();
                let txid: Txid = deserialize(&previous_tx_id_bytes).unwrap();

                let tx_bytes: Vec<u8> = row.get(1).unwrap();
                let tx: Transaction = deserialize(&tx_bytes).unwrap();

                Ok((txid, tx))
            })
            .unwrap();

        let mut previous_tx_ids: Vec<Txid> = Vec::new();
        let mut txs: Vec<Transaction> = Vec::new();

        for row in convenant_iter {
            let result = row.unwrap();
            let previous_txid = result.0;
            previous_tx_ids.push(previous_txid);
            let tx = result.1;
            txs.push(tx);
        }

        if txs.is_empty() {
            continue;
        }

        assert!(txs.len() == 1);
        assert!(previous_tx_ids.len() == 1);

        let satisfaction_weight = wallet
            .get_descriptor_for_keychain(KeychainKind::External)
            .max_satisfaction_weight()
            .unwrap();

        let tx = txs.get(0).unwrap();
        let previous_tx_id = previous_tx_ids.get(0).unwrap();

        return Some((previous_tx_id.clone(), tx.clone(), satisfaction_weight));
    }

    None
}
