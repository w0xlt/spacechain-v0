use std::fs::File;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bdk::{bitcoin::util::bip32::ExtendedPrivKey};
use bdk::keys::{GeneratedKey, bip39::{Mnemonic, WordCount, Language}, GeneratableKey};
use bdk::miniscript::Tap;

use std::io::{Write, Read};

fn write_birthdate(wallet_dir: &PathBuf) {

    let mut path = wallet_dir.clone();
    path.push("birthdate");

    let birthdate = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let mut file = File::create(path).unwrap();
    file.write_all(&birthdate.to_ne_bytes()).unwrap();
}

pub fn get_wallet_xpriv(wallet_name: &str, network: bdk::bitcoin::Network) -> Option<ExtendedPrivKey>
{
    let home_dir = dirs::home_dir();

    if home_dir == None {
        println!("Impossible to get the home dir!");
        return None;
    }

    let mut path = home_dir.unwrap();

    path.push(".spacechains");
    path.push(wallet_name);

    std::fs::create_dir_all(path.clone()).unwrap();

    let wallet_dir = path.clone();

    path.push("keystore");

    let path_str = path.as_os_str().to_str().unwrap();

    let key_file = File::open(path.clone()).ok();

    let xprv: Option<ExtendedPrivKey>;

    if let Some(mut file) = key_file {
        let mut buffer = Vec::<u8>::new();
        file.read_to_end(&mut buffer).unwrap();

        xprv = Some(ExtendedPrivKey::new_master(network, &buffer).unwrap());

    } else {

        println!("No wallet found in {}. Creating a new wallet ...", path_str);
        println!("Generating new seed.");

        let mnemonic: GeneratedKey<Mnemonic, Tap> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| bdk::Error::Generic("Mnemonic generation error".to_string())).unwrap();

        println!("Wallet mnemonic: {}", *mnemonic);

        let seed = mnemonic.to_seed("");

        let mut output = File::create(path).unwrap();
        output.write_all(&seed).unwrap();

        write_birthdate(&wallet_dir);

        xprv = Some(ExtendedPrivKey::new_master(network, &seed).unwrap());
    }

    xprv
}