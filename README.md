# Spacechains Mining Implementation

## Intro

This project reimplements the [spacechain proposal](https://github.com/RubenSomsen/spacechains) in BDK, improving the UX and automating all the steps.

For now, it deliberately keeps the same behavior as the original project.

## Requirements

To run this software, signet Bitcoin Core is required.

## Tutorial

### 0 - Clone the project and build

```
$ git clone git@github.com:w0xlt/spacechain-v0.git
$ git checkout -b add_bcore_rpc origin/add_bcore_rpc
$ cargo build --release
$ cd target
$ cd release
```

By default, a configuration file is created in `$HOME/.spacechains/spacechains.conf`.
When executing any comand below, the software will try to connect to `127.0.0.1:38332` using the authentication file `$HOME/.bitcoin/signet/.cookie`.
If these settings are different on the machine, the file should be edited.

### 1 - Create and fund the wallet

Then run the software with the following command to get a new address. If a wallet does not exist, this command will create one.

```
$ spacechains new_address

No wallet found in /home/node/.spacechains/default/keystore. Creating a new wallet ...
Generating new seed.
Wallet mnemonic: legend secret donor sense curtain tunnel aspect mother vacant cycle they harbor
{
  "address": "tb1qslstnfhvqxqz339vsmdsuarrezqlks2hcdxzw4",
  "index": 0
}
```

Note that the wallet was created in `$HOME/.spacechains`. The next folder `default` is the wallet name. `keystore` is the file in which the master key is stored.

Proceed to the [signet faucet](https://signet.bc-2.jp/) and send 0.01 coins on the address indicated in the terminal.

### 2 - Check wallet balance

The wallet balance can be checked with the following command:

```
$ spacechains balance
{
  "confirmed": 1000000,
  "immature": 0,
  "trusted_pending": 0,
  "untrusted_pending": 0
}
```

Wait until the balance is `confirmed` and proceed to the next step. This may take a while.

### 3 - Broadcast transactions

The `broadcast` command will create and broadcast the covenant and the fee-bumping CPFP transactions.

If broadcasted successfully, the command will return the id of both transactions.

```
$ spacechains broadcast 100000 "Hello World"

{
  "covenant_transaction_id": "0e5dbbb78236116f741399e617048d2ebc7e4c6b3d5038306afea4d776acd2a7",
  "cpfp_transaction_id": "1c5ec24460adf9d020d1556d15a571e24546e5ee2693c5c1b6dd12a9472a09c1"
}

```

The first parameter is the fee amount to be paid. This must be less than the confirmed wallet balance.

The second is the data to be include in the `OP_RETURN` output of the fee-bumping CPFP transaction.

### 4 - Backup wallet

The `backup` command shows the private descriptor. With this, the wallet can be exported to Bitcoin Core or others that support descriptors.


```
$ spacechains backup

{
  "blockheight": 116443,
  "descriptor": "wpkh(tprv8ZgxMBicQKsPfKxtuhjVE7kk7xj3RAbxY7axSoM6mUZYzfSvk6Ke31wTscSvffsdC3aU1js6ZLPMjwT3SgJ3duM5W8ReWLWs5Ad9UuwbCep/84'/1'/0'/0/*)",
  "label": "exported wallet"
}
```

### 5 - Config File

The command `config_file` displays the configurable parameters the user can customize.

The `blockchain` option can be `bitcoin_rpc` and `electrum`.

For now, only `signet` option is supported.

Other options can be changed according to user's Electrum, bitcoind and covenants settings.

The first line shows where the file is located. By default it is in `$HOME/.spacechains/spacechains.conf`

```
$ spacechains config_file

Config file located in /home/node/.spacechains/spacechains.conf
{
  "bitcoind_auth_file": "/home/node/.bitcoin/signet/.cookie",
  "bitcoind_url": "127.0.0.1:38332",
  "blockchain": "bitcoin_rpc",
  "covenant_genesis_tx": "60c31751818bd4410eed84b1c9047863206cce2c7d4d610ce5841c4195ba6c3b",
  "covenant_private_key_hex": "eb445ec7e0fd814db1e84622cddad9cd30154ee22bc6c2a4a61f6287be39f2d2",
  "electrum_url": "tcp://127.0.0.1:50001",
  "network": "signet"
}

```




