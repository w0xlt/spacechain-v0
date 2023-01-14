# Spacechains Mining Implementation

## Intro

This project reimplements the [spacechain proposal](https://github.com/RubenSomsen/spacechains) in BDK, improving the UX and automating all the steps.

For now, it deliberately keeps the same behavior as the original project.

## Requirements

To run this software, signet [Electrs server](https://github.com/romanz/electrs) running on `tcp://127.0.0.1:50001` is required.

## Tutorial

### 1 - Create and fund the wallet

```
$ git clone git@github.com:w0xlt/spacechain-v0.git
$ cargo build --release
$ cd target
$ cd release
```

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



