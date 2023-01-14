# Spacechains Mining Implementation

## Intro

This project reimplements the [spacechain proposal](https://github.com/RubenSomsen/spacechains).

Some improvements over the original design:

* The covenant's private key is no longer in the code. All covenant transaction are pre-signed and stored in `covenant.db`.

* Instead of using P2SH, all transactions are P2WSH, so they cannot be malleated.

* Instead of manually calling the Bitcoin-Core wallet, it uses BDK, improving the UX and automating all the steps.

## Caveats

Currently `covenant.db` is only configured for testnet covenant pre-signed transactions.

But a `covenant.db` for signet can be via [spacechain generator project](https://github.com/w0xlt/spacechains-generator).

*Do not generate it for mainnet. The code is not reviewed and will result in irreversible loss of funds.*

## Tutorial

### 0 - Clone the project and build

```
$ git clone git@github.com:w0xlt/spacechain-v0.git
$ cd spacechain-v0
$ cargo build --release
$ cd target
$ cd release
$ cp ../../convenant.db .
```

By default, a configuration file is created in `$HOME/.spacechains/spacechains.conf`.
When executing any comand below, the software will try to connect to the Blockstream's testnet server `ssl://electrum.blockstream.info:60002`.
If different settings are required, the file should be edited.

### 1 - Create or Import a wallet

To create a new wallet, run:

```
$ spacechains create-wallet w1

Wallet created successfully !
```

Alternatively, a wallet can be imported. This can be used when the user already has a funded wallet.

```
cargo run import-wallet "w2" "wpkh(tprv8ZgxMBicQKsPcyyLXWZPp4Xt7jKNDKvMZBhu195bqp2Hv2nvTEtziqpr4cZcJiEKdUD1AD9CoGUihrgmjoXukFbDEtLptcyinEbPQjouBhH/84'/1'/0'/0/*)#tqeruafe" "wpkh(tprv8ZgxMBicQKsPcyyLXWZPp4Xt7jKNDKvMZBhu195bqp2Hv2nvTEtziqpr4cZcJiEKdUD1AD9CoGUihrgmjoXukFbDEtLptcyinEbPQjouBhH/84'/1'/0'/1/*)#65uzpgep"
```

`w1` and `w2` are the wallet names. Any name can be used.

### 2 - Get a new address and fund it

Then run the software with the following command to get a new address. If a wallet does not exist, this command will create one.

```
$ spacechains get-new-address w1
{
  "address": "tb1qqe2xuqa4kt5j40tfd0m2820q9njxpd23tse2wp",
  "index": 0
}
```

Note that the wallet was created in `$HOME/.spacechains`. The keys of all wallets are stored in `$HOME/.spacechains/wallet.db`.

Proceed to the [testnet faucet](https://coinfaucet.eu/en/btc-testnet/) and send some coins on the address indicated in the terminal.

### 3 - Check wallet balance

The wallet balance can be checked with the following command:

```
$ spacechains get-balance w1
{
  "confirmed": 1349017,
  "immature": 0,
  "trusted_pending": 0,
  "untrusted_pending": 0
}
```

Wait until the balance is `confirmed` and proceed to the next step. This may take a while.

### 3 - Mine a new block

The `mine` command will create and broadcast the covenant and the fee-bumping CPFP transactions.

If broadcasted successfully, the command will return the id of both transactions.

```
$ spacechains mine w1 "Hello World" 100000

{
  "covenant_transaction_id": "0e5dbbb78236116f741399e617048d2ebc7e4c6b3d5038306afea4d776acd2a7",
  "cpfp_transaction_id": "1c5ec24460adf9d020d1556d15a571e24546e5ee2693c5c1b6dd12a9472a09c1"
}

```

The first is the data to be include in the `OP_RETURN` output of the fee-bumping CPFP transaction.

The second parameter is the fee amount to be paid. This must be less than the confirmed wallet balance.

### 4 - Backup wallet

The `backup` command shows the private descriptor. With this, the wallet can be exported to Bitcoin Core or others that support descriptors.


```
$ spacechains backup w1

{
  "blockheight": 116443,
  "descriptor": "wpkh(tprv8ZgxMBicQKsPcyyLXWZPp4Xt7jKNDKvMZBhu195bqp2Hv2nvTEtziqpr4cZcJiEKdUD1AD9CoGUihrgmjoXukFbDEtLptcyinEbPQjouBhH/84'/1'/0'/0/*)",
  "label": "exported wallet"
}
```

### 5 - Config File

The command `config_file` displays the configurable parameters the user can customize.

The `blockchain` option can be `bitcoin_rpc` and `electrum`.

For now, only `signet` and `testnet` options are supported.

Other options can be changed according to user's Electrum, bitcoind settings.

The first line shows where the file is located. By default it is in `$HOME/.spacechains/spacechains.conf`

```
$ spacechains config-file

Config file located in /home/node/.spacechains/spacechains.conf
{
  "bitcoind_auth_file": "/home/node/.bitcoin/signet/.cookie",
  "bitcoind_url": "127.0.0.1:38332",
  "blockchain": "electrum",
  "electrum_url": "ssl://electrum.blockstream.info:60002",
  "network": "testnet"
}
```




