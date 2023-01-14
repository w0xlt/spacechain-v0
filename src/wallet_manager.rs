use bdk::bitcoin::{psbt, Network, OutPoint, Script, Sequence, Transaction, Witness};
use bdk::database::SqliteDatabase;
use bdk::database::{any::SqliteDbConfiguration, ConfigurableDatabase};
use bdk::{SignOptions, Wallet};

use crate::utils;

pub fn load_wallet(
    external_descriptor: &String,
    internal_descriptor: &Option<String>,
    network: Network,
) -> Wallet<SqliteDatabase> {
    let path = utils::get_bdk_wallet_path(external_descriptor, internal_descriptor, network);

    let sqlite_db_configuration = SqliteDbConfiguration {
        path: path.into_os_string().into_string().unwrap(),
    };

    let sqlite_database =
        bdk::database::SqliteDatabase::from_config(&sqlite_db_configuration).unwrap();

    Wallet::new(
        external_descriptor,
        internal_descriptor.as_ref(),
        network,
        sqlite_database,
    )
    .unwrap()
}
pub fn create_cpfp_transaction(
    cpfp_wallet: &Wallet<SqliteDatabase>,
    output: &str,
    covenant_transaction: &Transaction,
    satisfaction_weight: usize,
    fee_amount: u64,
) -> Transaction {
    let balance = cpfp_wallet.get_balance().unwrap();

    if balance.confirmed == 0 || balance.confirmed < fee_amount {
        panic!("Insufficient funds !")
    }

    let bump_script = utils::build_bump_script().to_v0_p2wsh();

    let mut bump_tx_vout: Option<u32> = None;
    let mut bump_amount: u64 = 0;

    for (index, out) in covenant_transaction.output.iter().enumerate() {
        if out.script_pubkey == bump_script {
            bump_tx_vout = Some(index as u32);
            bump_amount = out.value;
        }
    }

    if bump_tx_vout == None {
        panic!("Bump script not found in the covenant transaction.");
    }

    let outpoint = OutPoint {
        txid: covenant_transaction.txid(),
        vout: bump_tx_vout.unwrap(),
    };

    let mut tx_builder = cpfp_wallet.build_tx();

    let bump_txout = covenant_transaction.output[bump_tx_vout.unwrap() as usize].clone();

    let x = utils::build_bump_script().as_bytes().to_vec();

    let psbt_input = psbt::Input {
        non_witness_utxo: Some(covenant_transaction.clone()),
        witness_utxo: Some(bump_txout),
        final_script_witness: Some(Witness::from_vec(vec![x])),
        ..Default::default()
    };

    tx_builder
        .add_foreign_utxo(outpoint, psbt_input, satisfaction_weight)
        .unwrap();

    let op_return_script = Script::new_op_return(output.as_bytes());

    tx_builder.add_recipient(op_return_script, bump_amount);

    tx_builder.fee_absolute(fee_amount);

    tx_builder.current_height(0);

    tx_builder.version(2);

    let (mut psbt, _) = tx_builder.finish().unwrap();

    for inp in psbt.unsigned_tx.input.iter_mut() {
        if inp.previous_output.txid == covenant_transaction.txid() {
            inp.sequence = Sequence::ZERO;
        }
    }

    cpfp_wallet.sign(&mut psbt, SignOptions::default()).unwrap();

    psbt.extract_tx()
}
