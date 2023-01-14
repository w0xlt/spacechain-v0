use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::DerivationPath;
use bdk::bitcoin::util::bip32::ExtendedPrivKey;
use bdk::bitcoin::Network;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    GeneratableKey, GeneratedKey,
};
use bdk::miniscript::Tap;
use rusqlite::{params, Connection};

use crate::utils;

pub fn create_new_wallet_desc(wallet_name: &str, network: Network) {
    let mnemonic: GeneratedKey<Mnemonic, Tap> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| bdk::Error::Generic("Mnemonic generation error".to_string()))
            .unwrap();

    let seed = mnemonic.to_seed("");

    let xprv = ExtendedPrivKey::new_master(network, &seed).unwrap();

    let external_path = DerivationPath::from_str("m/84h/0h/0h/0").unwrap();
    let internal_path = DerivationPath::from_str("m/84h/0h/0h/1").unwrap();

    let secp = Secp256k1::new();

    let (external_descriptor, ext_keymap) = bdk::descriptor!(wpkh((xprv.clone(), external_path)))
        .unwrap()
        .into_wallet_descriptor(&secp, network)
        .unwrap();

    let (internal_descriptor, int_keymap) = bdk::descriptor!(wpkh((xprv.clone(), internal_path)))
        .unwrap()
        .into_wallet_descriptor(&secp, network)
        .unwrap();

    let external_descriptor_str = external_descriptor.to_string_with_secret(&ext_keymap);

    let internal_descriptor_str = internal_descriptor.to_string_with_secret(&int_keymap);

    let birthdate = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let path = utils::get_keystore_db_path();

    write_wallet_data(
        &path,
        wallet_name,
        &external_descriptor_str,
        &Some(internal_descriptor_str),
        birthdate,
    );
}

pub fn import_wallet_desc(
    wallet_name: &str,
    external_descriptor: &String,
    internal_descriptor: &Option<String>,
) {
    let path = utils::get_keystore_db_path();

    let birthdate = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    write_wallet_data(
        &path,
        wallet_name,
        external_descriptor,
        internal_descriptor,
        birthdate,
    );
}

fn write_wallet_data(
    database_file: &PathBuf,
    wallet_name: &str,
    external_descriptor: &String,
    internal_descriptor: &Option<String>,
    birthdate: u64,
) {
    let conn = Connection::open(database_file).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS wallet_data (wallet_name TEXT UNIQUE NOT NULL, external_descriptor BLOB NOT NULL, internal_descriptor BLOB, birthdate INT NOT NULL);",
        [],
    )
    .unwrap();

    let mut internal_descriptor_data: Option<Vec<u8>> = None;
    if internal_descriptor.is_some() {
        let desc: Vec<u8> = (*internal_descriptor.clone().unwrap().as_bytes()).to_vec();
        internal_descriptor_data = Some(desc);
    }

    match conn.execute(
        "INSERT INTO wallet_data (wallet_name, external_descriptor, internal_descriptor, birthdate) VALUES (?1, ?2, ?3, ?4)",
        params![wallet_name, external_descriptor.as_bytes(), internal_descriptor_data, birthdate],
    ) {
        Ok(_) => {},
        Err(rusqlite::Error::SqliteFailure(rc, _ )) => {
            if rc.code == rusqlite::ErrorCode::ConstraintViolation {
                assert_eq!(rc.extended_code, 2067);
                panic!("There is already a wallet with the name {}. Please choose another name.", wallet_name);
            }

        },
        Err(err) => panic!("{}", err)
    }
}

pub fn load_descriptors(
    database_file: &PathBuf,
    wallet_name: &String,
) -> Option<(String, Option<String>, u64)> {
    let conn = Connection::open(database_file).unwrap();

    let mut stmt = conn.prepare("SELECT external_descriptor, internal_descriptor, birthdate FROM wallet_data WHERE wallet_name = ?1").unwrap();

    let convenant_iter = stmt
        .query_map([wallet_name], |row| {
            let external_descriptor: Vec<u8> = row.get(0).unwrap();
            let internal_descriptor: Option<Vec<u8>> = row.get(1).unwrap();
            let birthdate: u64 = row.get(2).unwrap();
            Ok((external_descriptor, internal_descriptor, birthdate))
        })
        .unwrap();

    let mut wallet_data: Vec<(String, Option<String>, u64)> = Vec::new();

    for row in convenant_iter {
        let result = row.unwrap();

        let external_descriptor = String::from_utf8(result.0).unwrap();

        let mut internal_descriptor: Option<String> = None;
        if result.1.is_some() {
            internal_descriptor = Some(String::from_utf8(result.1.unwrap()).unwrap());
        }

        let birthdate = result.2;

        wallet_data.push((external_descriptor, internal_descriptor, birthdate));
    }

    if wallet_data.len() == 1 {
        Some(wallet_data.get(0).unwrap().clone())
    } else {
        assert_eq!(wallet_data.len(), 0);
        None
    }
}
