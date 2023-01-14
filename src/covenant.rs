use bdk::bitcoin::{Txid, Script, Transaction, Network, PrivateKey};
use bdk::bitcoin::secp256k1::{Secp256k1, SecretKey, PublicKey};
use bdk::database::SqliteDatabase;
use bdk::miniscript::Descriptor;
use bdk::database::{any::SqliteDbConfiguration, ConfigurableDatabase};
use bdk::{Wallet, TransactionDetails, LocalUtxo, SignOptions, KeychainKind};

use std::str::FromStr;

use crate::utils;

pub fn create_convenant_transaction(network: bdk::bitcoin::Network) -> (Transaction, usize)
{
    let wallet = load_convenant_wallet(network);
    let utxo = get_convenant_utxo(&wallet, network).unwrap();
    let tx = build_transaction(&wallet, &utxo, network);

    // This is used to add this convenant_transaction as input in the cpfp_transaction
    let satisfaction_weight = wallet
            .get_descriptor_for_keychain(KeychainKind::External)
            .max_satisfaction_weight()
            .unwrap();

    (tx, satisfaction_weight)

    // println!("COVENANT UTXO: {}:{}", utxo.outpoint.txid, utxo.outpoint.vout);
    // let consensus_encoded = serialize(&tx);

    // println!("Covenant tx: {:02x?}", consensus_encoded);
}

fn covenant_descriptor(private_key: bool, network: bdk::bitcoin::Network) -> String
{
    const KEY_STRING: &str = "eb445ec7e0fd814db1e84622cddad9cd30154ee22bc6c2a4a61f6287be39f2d2";

    let sk_vec = hex::decode(KEY_STRING).unwrap();
    let sk = SecretKey::from_slice(&sk_vec).unwrap();

    let key: String;

    if private_key {
        let prv= PrivateKey::from_slice(&sk_vec, network).unwrap();
        key = PrivateKey::to_wif(prv);
    } else {
        let curve = Secp256k1::new();
        let pk = PublicKey::from_secret_key(&curve, &sk);
        key = hex::encode(pk.serialize());
    }

    format!("sh(and_v(v:pk({}),older(1)))", key)
}

fn load_convenant_wallet(network: bdk::bitcoin::Network) -> Wallet<SqliteDatabase>
{
    let home_dir = dirs::home_dir();

    let mut path = home_dir.unwrap();

    path.push(".spacechains");
    path.push("convenat");

    std::fs::create_dir_all(path.clone()).unwrap();

    path.push("database");

    let sqlite_db_configuration = SqliteDbConfiguration{path: path.into_os_string().into_string().unwrap()};

    let sqlite_database = bdk::database::SqliteDatabase::from_config(&sqlite_db_configuration).unwrap();

    let desc = covenant_descriptor(true, network);

    Wallet::new(desc.as_str(), None, Network::Signet, sqlite_database).unwrap()
}

fn covenant_tx_validation(tx: &Transaction, covenant_script_pubkey: &Script, bump_script: &Script) -> bool
{
    if tx.input.len() != 1 {
        return false;
    }

    if tx.output.len() != 2 {
        return false;
    }

    let tx_out_0 = tx.output.get(0).unwrap();
    let tx_out_1 = tx.output.get(1).unwrap();

    if tx_out_0.script_pubkey != covenant_script_pubkey.to_p2sh() && tx_out_1.script_pubkey != covenant_script_pubkey.to_p2sh() {
        return false;
    }

    if tx_out_0.script_pubkey != bump_script.to_p2sh() && tx_out_1.script_pubkey != bump_script.to_p2sh() {
        return false;
    }

    true
}

fn check_covenant(
    tx_list: &Vec<TransactionDetails>,
    txid: &Txid,
    covenant_script_pubkey: &Script,
    bump_script: &Script) -> (bool, u32)
{
    const ORIGINAL_COVENANT_TX: &str = "60c31751818bd4410eed84b1c9047863206cce2c7d4d610ce5841c4195ba6c3b";

    let mut tx_details = tx_list.iter().find(
        |tx| tx.txid == *txid
    ).unwrap();
    let mut tx = tx_details.transaction.clone().unwrap();

    let height = tx_details.confirmation_time.clone().unwrap_or_default().height;

    if height == 0 {
        panic!("The last transaction of the covenant {} has not yet been confirmed. Please wait for at least one confirmation!", tx.txid());
    }

    loop {
        if !covenant_tx_validation(&tx, covenant_script_pubkey, bump_script) {
            break;
        }

        let tx_in_0 = tx.input.get(0).unwrap();

        tx_details = tx_list.iter().find(
            |tx| tx.txid == tx_in_0.previous_output.txid
        ).unwrap();
        tx = tx_details.transaction.clone().unwrap();
    }

    (tx.txid().to_string() == ORIGINAL_COVENANT_TX, height)
}

fn get_convenant_utxo(wallet: &Wallet<SqliteDatabase>, network: bdk::bitcoin::Network) -> Option<LocalUtxo>
{
    let desc_str = covenant_descriptor(false, network);
    let desc = Descriptor::<bdk::bitcoin::PublicKey>::from_str(desc_str.as_str()).unwrap();
    let covenant_script_pubkey = desc.script_code().unwrap();

    let bump_script = utils::build_bump_script(false);

    // let electrum_url = "tcp://127.0.0.1:50001";

    // let blockchain = ElectrumBlockchain::from(Client::new(electrum_url).unwrap());

    // wallet.sync(&blockchain, SyncOptions::default()).unwrap();

    utils::sync_wallet(&wallet);

    let tx_list = wallet.list_transactions(true).unwrap();

    let mut utxo_height: u32 = 0;
    let mut covenant_utxo: Option<LocalUtxo> = None;

    for utxo in wallet.list_unspent().unwrap().iter() {
        let valid_utxo = check_covenant(&tx_list, &utxo.outpoint.txid, &covenant_script_pubkey, &bump_script);

        if valid_utxo.0 && valid_utxo.1 > utxo_height {
            utxo_height = valid_utxo.1;
            covenant_utxo = Some(utxo.clone())
        }
    }

    covenant_utxo
}

fn build_transaction(wallet: &Wallet<SqliteDatabase>, utxo: &LocalUtxo, network: bdk::bitcoin::Network) -> Transaction
{
    let input_satoshis = utxo.txout.value;

    let (dust_limit, fee_amount) = (800, 1200);

    let mut tx_builder = wallet.build_tx();

    tx_builder.add_utxo(utxo.outpoint).unwrap();

    tx_builder.manually_selected_only();

    tx_builder.current_height(0);

    tx_builder.fee_absolute(fee_amount);

    let desc_str = covenant_descriptor(false, network);
    let desc = Descriptor::<bdk::bitcoin::PublicKey>::from_str(desc_str.as_str()).unwrap();
    let covenant_script_pubkey = desc.script_code().unwrap();

    let bump_script = utils::build_bump_script(false);

    let covenant_amount = input_satoshis - dust_limit - fee_amount;

    let bump_amount = dust_limit;

    tx_builder.set_recipients(vec![(bump_script.to_p2sh(), bump_amount), (covenant_script_pubkey.to_p2sh(), covenant_amount)]);

    let (mut psbt, _) = tx_builder.finish().unwrap();

    wallet.sign(&mut psbt, SignOptions::default()).unwrap();

    psbt.extract_tx()
}