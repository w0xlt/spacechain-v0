use std::str::FromStr;

use bdk::bitcoin::{self, Network};
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub network: String,
    pub blockchain: String,
    pub electrum_url: String,
    pub bitcoind_url: String,
    pub bitcoind_auth_file: String,
    pub covenant_private_key_hex: String,
    pub covenant_genesis_tx: String,
}

impl ConfigFile {
    #[allow(dead_code)]
    pub fn get_network(&self) -> Result<Network, std::string::String> {
        if self.network == "signet" {
            return Ok(bitcoin::Network::Signet);
        }
        Err("Only signet supported for now".to_string())
    }
}

pub fn create_or_get_default() -> (ConfigFile, String)
{
    let home_dir = dirs::home_dir();

    let mut path = home_dir.clone().unwrap();
    path.push(".spacechains");

    std::fs::create_dir_all(path.clone()).unwrap();

    path.push("spacechains.conf");

    let binding = path.clone();
    let path_str = binding.as_os_str().to_str().unwrap();

    let mut bc_path = home_dir.unwrap();
    bc_path.push(".bitcoin");
    bc_path.push("signet");
    bc_path.push(".cookie");

    let bc_path_str = bc_path.as_os_str().to_str().unwrap();

    if !path.exists() {
        let cfg = ConfigFile {
            network: String::from_str("signet").unwrap(),
            blockchain: String::from_str("electrum").unwrap(),
            electrum_url: String::from_str("tcp://127.0.0.1:50001").unwrap(),
            covenant_private_key_hex: String::from_str("eb445ec7e0fd814db1e84622cddad9cd30154ee22bc6c2a4a61f6287be39f2d2").unwrap(),
            bitcoind_url: "127.0.0.1:38332".to_string(),
            bitcoind_auth_file: bc_path_str.to_string(),
            covenant_genesis_tx: String::from_str("60c31751818bd4410eed84b1c9047863206cce2c7d4d610ce5841c4195ba6c3b").unwrap(),
        };

        confy::store_path(path, &cfg).unwrap();

        (cfg, String::from_str(path_str).unwrap())
    } else {
        let cfg: ConfigFile = confy::load_path(path).unwrap();

        (cfg, String::from_str(path_str).unwrap())
    }
}