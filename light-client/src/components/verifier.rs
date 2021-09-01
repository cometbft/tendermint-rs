//! Provides an interface and default implementation of the `Verifier` component

use crate::operations::voting_power::VotingPowerTally;
use crate::predicates as preds;
use crate::types::LightBlockState;
use crate::{
    errors::ErrorExt,
    light_client::Options,
    operations::{
        CommitValidator, Hasher, ProdCommitValidator, ProdHasher, ProdVotingPowerCalculator,
        VotingPowerCalculator,
    },
    types::Time,
};
use preds::{
    errors::{VerificationError, VerificationErrorDetail},
    ProdPredicates, VerificationPredicates,
};
use serde::{Deserialize, Serialize};

/// Represents the result of the verification performed by the
/// verifier component.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Verdict {
    /// Verification succeeded, the block is valid.
    Success,
    /// The minimum voting power threshold is not reached,
    /// the block cannot be trusted yet.
    NotEnoughTrust(VotingPowerTally),
    /// Verification failed, the block is invalid.
    Invalid(VerificationErrorDetail),
}

impl From<Result<(), VerificationError>> for Verdict {
    fn from(result: Result<(), VerificationError>) -> Self {
        match result {
            Ok(()) => Self::Success,
            Err(VerificationError(e, _)) => match e.not_enough_trust() {
                Some(tally) => Self::NotEnoughTrust(tally),
                _ => Self::Invalid(e),
            },
        }
    }
}

/// The verifier checks:
///
/// a) whether a given untrusted light block is valid, and
/// b) whether a given untrusted light block should be trusted
///    based on a previously verified block.
///
/// ## Implements
/// - [TMBC-VAL-CONTAINS-CORR.1]
/// - [TMBC-VAL-COMMIT.1]
pub trait Verifier: Send + Sync {
    /// Perform the verification.
    fn verify(
        &self,
        untrusted: LightBlockState<'_>,
        trusted: LightBlockState<'_>,
        options: &Options,
        now: Time,
    ) -> Verdict;
}

macro_rules! verdict {
    ($e:expr) => {
        let result = $e;
        if result.is_err() {
            return result.into();
        }
    };
}

/// Predicate verifier encapsulating components necessary to facilitate
/// verification.
pub struct PredicateVerifier<P, C, V, H> {
    predicates: P,
    voting_power_calculator: C,
    commit_validator: V,
    hasher: H,
}

impl<P, C, V, H> Default for PredicateVerifier<P, C, V, H>
where
    P: Default,
    C: Default,
    V: Default,
    H: Default,
{
    fn default() -> Self {
        Self {
            predicates: P::default(),
            voting_power_calculator: C::default(),
            commit_validator: V::default(),
            hasher: H::default(),
        }
    }
}

impl<P, C, V, H> PredicateVerifier<P, C, V, H>
where
    P: VerificationPredicates,
    C: VotingPowerCalculator,
    V: CommitValidator,
    H: Hasher,
{
    /// Constructor.
    pub fn new(predicates: P, voting_power_calculator: C, commit_validator: V, hasher: H) -> Self {
        Self {
            predicates,
            voting_power_calculator,
            commit_validator,
            hasher,
        }
    }
}

impl<P, C, V, H> Verifier for PredicateVerifier<P, C, V, H>
where
    P: VerificationPredicates,
    C: VotingPowerCalculator,
    V: CommitValidator,
    H: Hasher,
{
    /// Validate the given light block.
    ///
    /// - Ensure the latest trusted header hasn't expired
    /// - Ensure the header validator hashes match the given validators
    /// - Ensure the header next validator hashes match the given next
    ///   validators
    /// - Additional implementation specific validation via `commit_validator`
    /// - Check that the untrusted block is more recent than the trusted state
    /// - If the untrusted block is the very next block after the trusted block,
    ///   check that their (next) validator sets hashes match.
    /// - Otherwise, ensure that the untrusted block has a greater height than
    ///   the trusted block.
    fn verify(
        &self,
        untrusted: LightBlockState<'_>,
        trusted: LightBlockState<'_>,
        options: &Options,
        now: Time,
    ) -> Verdict {
        // Ensure the latest trusted header hasn't expired
        verdict!(self.predicates.is_within_trust_period(
            &trusted.signed_header.header,
            options.trusting_period,
            now,
        ));

        // Ensure the header isn't from a future time
        verdict!(self.predicates.is_header_from_past(
            &untrusted.signed_header.header,
            options.clock_drift,
            now,
        ));

        // Ensure the header validator hashes match the given validators
        verdict!(self.predicates.validator_sets_match(
            untrusted.validators,
            untrusted.signed_header.header.validators_hash,
            &self.hasher,
        ));

        // Ensure the header next validator hashes match the given next validators
        verdict!(self.predicates.next_validators_match(
            untrusted.next_validators,
            untrusted.signed_header.header.next_validators_hash,
            &self.hasher,
        ));

        // Ensure the header matches the commit
        verdict!(self.predicates.header_matches_commit(
            &untrusted.signed_header.header,
            untrusted.signed_header.commit.block_id.hash,
            &self.hasher,
        ));

        // Additional implementation specific validation
        verdict!(self.predicates.valid_commit(
            untrusted.signed_header,
            untrusted.validators,
            &self.commit_validator,
        ));

        // Check that the untrusted block is more recent than the trusted state
        verdict!(self.predicates.is_monotonic_bft_time(
            &untrusted.signed_header.header,
            &trusted.signed_header.header,
        ));

        let trusted_next_height = trusted.height().increment();

        if untrusted.height() == trusted_next_height {
            // If the untrusted block is the very next block after the trusted block,
            // check that their (next) validator sets hashes match.
            verdict!(self.predicates.valid_next_validator_set(
                &untrusted.signed_header.header,
                &trusted.signed_header.header,
            ));
        } else {
            // Otherwise, ensure that the untrusted block has a greater height than
            // the trusted block.
            verdict!(self.predicates.is_monotonic_height(
                &untrusted.signed_header.header,
                &trusted.signed_header.header,
            ));

            // Check there is enough overlap between the validator sets of
            // the trusted and untrusted blocks.
            verdict!(self.predicates.has_sufficient_validators_overlap(
                untrusted.signed_header,
                trusted.next_validators,
                &options.trust_threshold,
                &self.voting_power_calculator,
            ));
        }

        // Verify that more than 2/3 of the validators correctly committed the block.
        verdict!(self.predicates.has_sufficient_signers_overlap(
            untrusted.signed_header,
            untrusted.validators,
            &self.voting_power_calculator,
        ));

        Verdict::Success
    }
}

/// The default production implementation of the [`PredicateVerifier`].
pub type ProdVerifier =
    PredicateVerifier<ProdPredicates, ProdVotingPowerCalculator, ProdCommitValidator, ProdHasher>;
