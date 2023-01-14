use std::fs::File;

use bdk::{bitcoin::util::bip32::ExtendedPrivKey};
use bdk::keys::{GeneratedKey, bip39::{Mnemonic, WordCount, Language}, GeneratableKey};
use bdk::miniscript::Tap;

use std::io::{Write, Read};

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

    match std::fs::create_dir_all(path.clone()) {
        Ok(_) => {},
        Err(_) => {
            println!("Unable to create or locate storage dir!");
            return None;
        }
    }

    path.push("keystore");

    let mut key_file: Option<File>= None;

    let path_str = path.as_os_str().to_str().unwrap();

    match File::open(path.clone()) {
        Ok(file) => {
            key_file = Some(file);
        },
        Err(_) => { }
    }

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

        xprv = Some(ExtendedPrivKey::new_master(network, &seed).unwrap());
    }

    xprv
}