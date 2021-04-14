use crate::{
    transaction::Error,
    utils::{io, key_parser::read_ed25519_secret_key_from_file},
};
use bech32::{self, ToBase32 as _};
use chain_core::property::Serialize as _;
use chain_impl_mockchain::{
    account::SpendingCounter,
    header::HeaderId,
    transaction::{TransactionSignDataHash, Witness},
};
use std::{io::Write, path::PathBuf};
#[cfg(feature = "structopt")]
use structopt::StructOpt;

#[cfg_attr(
    feature = "structopt",
    derive(StructOpt),
    structopt(rename_all = "kebab-case")
)]
pub struct MkWitness {
    /// the Transaction ID of the witness to sign
    #[cfg_attr(feature = "structopt", structopt(name = "TRANSACTION_ID"))]
    pub sign_data_hash: TransactionSignDataHash,

    /// the file path to the file to write the witness in.
    /// If omitted it will be printed to the standard output.
    pub output: Option<PathBuf>,

    /// the type of witness to build: account, UTxO or Legacy UtxO
    #[cfg_attr(feature = "structopt", structopt(long = "type", parse(try_from_str)))]
    pub witness_type: WitnessType,

    /// the hash of the block0, the first block of the blockchain
    #[cfg_attr(
        feature = "structopt",
        structopt(long = "genesis-block-hash", parse(try_from_str))
    )]
    pub genesis_block_hash: HeaderId,

    /// value is mandatory is `--type=account' It is the counter for
    /// every time the account is being utilized.
    #[cfg_attr(feature = "structopt", structopt(long = "account-spending-counter"))]
    pub account_spending_counter: Option<u32>,

    /// the file path to the file to read the signing key from.
    /// If omitted it will be read from the standard input.
    pub secret: Option<PathBuf>,
}

pub enum WitnessType {
    UTxO,
    OldUTxO,
    Account,
}

impl std::str::FromStr for WitnessType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utxo" => Ok(WitnessType::UTxO),
            "legacy-utxo" => Ok(WitnessType::OldUTxO),
            "account" => Ok(WitnessType::Account),
            _ => Err("Invalid witness type, expected `utxo', `legacy-utxo' or `account'"),
        }
    }
}

impl MkWitness {
    pub fn exec(self) -> Result<(), Error> {
        let witness = match self.witness_type {
            WitnessType::UTxO => {
                let secret_key = read_ed25519_secret_key_from_file(&self.secret)?;
                Witness::new_utxo(&self.genesis_block_hash, &self.sign_data_hash, |d| {
                    secret_key.sign(d)
                })
            }
            WitnessType::OldUTxO => {
                let secret_key = read_ed25519_secret_key_from_file(&self.secret)?;
                Witness::new_old_utxo(
                    &self.genesis_block_hash,
                    &self.sign_data_hash,
                    |d| (secret_key.to_public(), secret_key.sign(d)),
                    &[0; 32],
                )
            }
            WitnessType::Account => {
                let account_spending_counter = self
                    .account_spending_counter
                    .ok_or(Error::MakeWitnessAccountCounterMissing)
                    .map(SpendingCounter::from)?;

                let secret_key = read_ed25519_secret_key_from_file(&self.secret)?;
                Witness::new_account(
                    &self.genesis_block_hash,
                    &self.sign_data_hash,
                    account_spending_counter,
                    |d| secret_key.sign(d),
                )
            }
        };

        self.write_witness(&witness)
    }

    fn write_witness(&self, witness: &Witness) -> Result<(), Error> {
        let mut writer =
            io::open_file_write(&self.output).map_err(|source| Error::WitnessFileWriteFailed {
                source,
                path: self.output.clone().unwrap_or_default(),
            })?;
        let bytes = witness
            .serialize_as_vec()
            .map_err(Error::WitnessFileSerializationFailed)?;

        let base32 = bytes.to_base32();
        let bech32 = bech32::encode("witness", &base32)?;
        writeln!(writer, "{}", bech32).map_err(|source| Error::WitnessFileWriteFailed {
            source,
            path: self.output.clone().unwrap_or_default(),
        })
    }
}
