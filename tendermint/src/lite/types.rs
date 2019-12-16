//! All traits that are necessary and need to be implemented to use the main
//! verification logic in `super::verifier` for a light client.

// TODO can we abstract this away and use a generic identifier instead ?
// Ie. something that just implements Eq ?
// (Ismail): a really easy solution would be have a trait that expects an
// as_bytes(&self) -> &[u8] method. It's unlikely that a hash won't be
// representable as bytes, or an Id (that is basically also a hash)
// but this feels a a bit like cheating
use crate::account::Id;

use crate::block::Height;
use crate::Hash;

use failure::_core::fmt::Debug;
use std::time::SystemTime;

/// TrustedState stores the latest state trusted by a lite client,
/// including the last header and the validator set to use to verify
/// the next header.
pub struct TrustedState<H, V>
where
    H: Header,
    V: ValidatorSet,
{
    pub last_header: H, // height H-1
    pub validators: V,  // height H
}

/// SignedHeader bundles a Header and a Commit for convenience.
pub trait SignedHeader {
    type Header: Header;
    type Commit: Commit;

    fn header(&self) -> &Self::Header;
    fn commit(&self) -> &Self::Commit;
}

/// Header contains meta data about the block -
/// the height, the time, the hash of the validator set
/// that should sign this header, and the hash of the validator
/// set that should sign the next header.
pub trait Header: Debug {
    /// The header's notion of (bft-)time.
    /// We assume it can be converted to SystemTime.
    type Time: Into<SystemTime>;

    fn height(&self) -> Height;
    fn bft_time(&self) -> Self::Time;
    fn validators_hash(&self) -> Hash;
    fn next_validators_hash(&self) -> Hash;

    /// Hash of the header (ie. the hash of the block).
    fn hash(&self) -> Hash;
}

/// ValidatorSet is the full validator set.
/// It exposes its hash, which should match whats in a header,
/// and its total power. It also has an underlying
/// Validator type which can be used for verifying signatures.
/// It also provides a lookup method to fetch a validator by
/// its identifier.
pub trait ValidatorSet {
    type Validator: Validator;

    /// Hash of the validator set.
    fn hash(&self) -> Hash;

    /// Total voting power of the set
    fn total_power(&self) -> u64;

    /// Fetch validator via their ID (ie. their address).
    fn validator(&self, val_id: Id) -> Option<Self::Validator>;

    /// Return the number of validators in this validator set.
    fn len(&self) -> usize;

    /// Returns true iff the validator set is empty.
    fn is_empty(&self) -> bool;
}

/// Validator has a voting power and can verify
/// its own signatures. Note it must have implicit access
/// to its public key material to verify signatures.
pub trait Validator {
    fn power(&self) -> u64;
    fn verify_signature(&self, sign_bytes: &[u8], signature: &[u8]) -> bool;
}

/// Commit is proof a Header is valid.
/// It has an underlying Vote type with the relevant vote data
/// for verification.
pub trait Commit {
    /// Hash of the header this commit is for.
    fn header_hash(&self) -> Hash;

    /// Compute the voting power of the validators that correctly signed the commit,
    /// have according to their voting power in the passed in validator set.
    /// Will return an error in case an invalid signature was included.
    ///
    /// This method corresponds to the (pure) auxiliary function int the spec:
    /// `votingpower_in(signers(h.Commit),h.Header.V)`.
    fn voting_power_in<V>(&self, vals: &V) -> Result<u64, Error>
    where
        V: ValidatorSet;

    /// Return the number of votes included in this commit
    /// (including nil/empty votes).
    fn votes_len(&self) -> usize;
}

/// TrustThreshold defines what fraction of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
/// The default implementation returns true, iff at least a third of the trusted
/// voting power signed (in other words at least one honest validator signed).
/// Some clients might require more than +1/3 and can implement their own
/// TrustLevel which can be passed into all relevant methods.
pub trait TrustThreshold {
    fn is_enough_power(&self, signed_voting_power: u64, total_voting_power: u64) -> bool {
        signed_voting_power * 3 > total_voting_power
    }
}

#[derive(Debug)]
pub enum Error {
    Expired,
    DurationOutOfRange,
    NonSequentialHeight,
    NonIncreasingHeight,

    InvalidValidatorSet,
    InvalidNextValidatorSet,
    InvalidCommitValue, // commit is not for the header we expected
    InvalidCommitLength,
    InvalidSignature,

    InsufficientVotingPower,
}
