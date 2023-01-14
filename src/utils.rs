use bdk::{bitcoin::{Transaction, blockdata::{script, opcodes}, Script}, Wallet, database::SqliteDatabase, SyncOptions};
use bdk::blockchain::{ElectrumBlockchain, Blockchain};
use bdk::electrum_client::Client;

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

pub fn sync_wallet(wallet: &Wallet<SqliteDatabase>)
{
    let electrum_url = "tcp://127.0.0.1:50001";

    let blockchain = ElectrumBlockchain::from(Client::new(electrum_url).unwrap());

    wallet.sync(&blockchain, SyncOptions::default()).unwrap();
}