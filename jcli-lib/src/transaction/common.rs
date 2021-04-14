use crate::transaction::{staging::Staging, Error};
use chain_impl_mockchain::fee::{LinearFee, PerCertificateFee, PerVoteCertificateFee};
use std::{num::NonZeroU64, path::PathBuf};
#[cfg(feature = "structopt")]
use structopt::StructOpt;

#[cfg_attr(
    feature = "structopt",
    derive(StructOpt),
    structopt(rename_all = "kebab-case")
)]
pub struct CommonFees {
    /// fee per transaction
    #[cfg_attr(
        feature = "structopt",
        structopt(long = "fee-constant", default_value = "0")
    )]
    pub constant: u64,
    /// fee per every input and output
    #[cfg_attr(
        feature = "structopt",
        structopt(long = "fee-coefficient", default_value = "0")
    )]
    pub coefficient: u64,
    /// fee per certificate
    #[cfg_attr(
        feature = "structopt",
        structopt(long = "fee-certificate", default_value = "0")
    )]
    pub certificate: u64,
    /// fee per pool registration (default: fee-certificate)
    #[cfg_attr(feature = "structopt", structopt(long = "fee-pool-registration"))]
    pub certificate_pool_registration: Option<u64>,
    /// fee per stake delegation (default: fee-certificate)
    #[cfg_attr(feature = "structopt", structopt(long = "fee-stake-delegation"))]
    pub certificate_stake_delegation: Option<u64>,
    /// fee per owner stake delegation (default: fee-certificate)
    #[cfg_attr(feature = "structopt", structopt(long = "fee-owner-stake-delegation"))]
    pub certificate_owner_stake_delegation: Option<u64>,
    /// fee per vote plan
    #[cfg_attr(feature = "structopt", structopt(long = "fee-vote-plan"))]
    pub certificate_vote_plan: Option<u64>,
    /// fee per vote cast
    #[cfg_attr(feature = "structopt", structopt(long = "fee-vote-cast"))]
    pub certificate_vote_cast: Option<u64>,
}

#[cfg_attr(
    feature = "structopt",
    derive(StructOpt),
    structopt(rename_all = "kebab-case")
)]
pub struct CommonTransaction {
    /// place where the transaction is going to be save during its staging phase
    /// If a file is given, the transaction will be read from this file and
    /// modification will be written into this same file.
    /// If no file is given, the transaction will be read from the standard
    /// input and will be rendered in the standard output
    #[cfg_attr(
        feature = "structopt",
        structopt(long = "staging", alias = "transaction")
    )]
    pub staging_file: Option<PathBuf>,
}

impl CommonFees {
    pub fn linear_fee(&self) -> LinearFee {
        let mut fees = LinearFee::new(self.constant, self.coefficient, self.certificate);
        let per_certificate_fees = PerCertificateFee::new(
            self.certificate_pool_registration.and_then(NonZeroU64::new),
            self.certificate_stake_delegation.and_then(NonZeroU64::new),
            self.certificate_owner_stake_delegation
                .and_then(NonZeroU64::new),
        );
        let per_vote_certificate_fees = PerVoteCertificateFee::new(
            self.certificate_vote_plan.and_then(NonZeroU64::new),
            self.certificate_vote_cast.and_then(NonZeroU64::new),
        );
        fees.per_certificate_fees(per_certificate_fees);
        fees.per_vote_certificate_fees(per_vote_certificate_fees);
        fees
    }
}

impl CommonTransaction {
    pub fn load(&self) -> Result<Staging, Error> {
        Staging::load(&self.staging_file)
    }

    pub fn store(&self, staging: &Staging) -> Result<(), Error> {
        staging.store(&self.staging_file)
    }
}
