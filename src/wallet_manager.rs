use bdk::bitcoin::{Transaction, Script, OutPoint, psbt, Sequence};
use bdk::database::SqliteDatabase;
use bdk::template::Bip84;
use bdk::wallet::AddressIndex;
use bdk::{bitcoin::util::bip32::ExtendedPrivKey};
use bdk::database::{any::SqliteDbConfiguration, ConfigurableDatabase};
use bdk::{Wallet, KeychainKind, SignOptions};

use crate::utils;

pub fn load_wallet(wallet_name: &str, xprv: &ExtendedPrivKey, network: bdk::bitcoin::Network) -> Wallet<SqliteDatabase>
{
    let mut path = dirs::home_dir().unwrap();

    path.push(".spacechains");
    path.push(wallet_name);

    if !path.exists() {
        panic!("Wallet dir does not exists!");
    }

    path.push("database");

    let sqlite_db_configuration = SqliteDbConfiguration{path: path.into_os_string().into_string().unwrap()};

    let sqlite_database = bdk::database::SqliteDatabase::from_config(&sqlite_db_configuration).unwrap();

    Wallet::new(
        Bip84(*xprv, KeychainKind::External),
        Some(Bip84(*xprv, KeychainKind::Internal)),
        network,
        sqlite_database,
    )
    .unwrap()

    // let addr = wallet.get_address(AddressIndex::New).unwrap();

    // println!("address: {}", addr.address.to_string());
    // println!("addr.index: {}", addr.index);
}

pub fn create_cpfp_transaction(
    wallet: &Wallet<SqliteDatabase>,
    output: &str,
    convenant_transaction: &Transaction,
    satisfaction_weight: usize,
    fee_amount: u64) -> Transaction
{
    utils::sync_wallet(wallet);

    let balance = wallet.get_balance().unwrap();

    if balance.confirmed == 0 || balance.confirmed < fee_amount {
        panic!("Insufficient funds !")
    }

    let bump_script = utils::build_bump_script(false).to_p2sh();

    let mut bump_tx_vout: Option<u32> = None;
    let mut bump_amount: u64 = 0;

    for (index, out) in convenant_transaction.output.iter().enumerate()  {
        if out.script_pubkey == bump_script {
            bump_tx_vout = Some(index as u32);
            bump_amount = out.value;
        }
    }

    if bump_tx_vout == None {
        panic!("Bump script not found in the covenant transaction.");
    }

    let outpoint = OutPoint {
        txid: convenant_transaction.txid(),
        vout: bump_tx_vout.unwrap()
    };

    let mut tx_builder = wallet.build_tx();

    let psbt_input = psbt::Input {
        non_witness_utxo: Some(convenant_transaction.clone()),
        redeem_script: Some(utils::build_bump_script(true)),
        final_script_sig: Some(utils::build_bump_script(true)),
        ..Default::default()
    };

    tx_builder.add_foreign_utxo(outpoint, psbt_input, satisfaction_weight).unwrap();

    let op_return_script = Script::new_op_return(output.as_bytes());

    tx_builder.add_recipient(op_return_script, bump_amount);

    let amount = balance.confirmed - fee_amount;

    let addr_info = wallet.get_address(AddressIndex::New).unwrap();
    tx_builder.add_recipient(addr_info.script_pubkey(), amount);

    tx_builder.fee_absolute(fee_amount);

    tx_builder.current_height(0);

    tx_builder.version(2);

    let (mut psbt, _) = tx_builder.finish().unwrap();

    for inp in psbt.unsigned_tx.input.iter_mut() {
        if inp.previous_output.txid == convenant_transaction.txid() {
            inp.sequence = Sequence::ZERO;
        }
    }

    wallet.sign(&mut psbt, SignOptions::default()).unwrap();

    psbt.extract_tx()
}