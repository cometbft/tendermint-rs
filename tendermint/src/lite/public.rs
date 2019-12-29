use crate::block::Height;
use crate::lite::*;
use std::time::{Duration, SystemTime};

// Verify a single untrusted header against a trusted state.
// Includes all validation and signature verification.
// Not publicly exposed since it does not check for expiry
// and hence it's possible to use it incorrectly.
// If trusted_state is not expired and this returns Ok, the
// untrusted_sh and untrusted_next_vals can be considered trusted.
fn verify_single<TS, SH, C, L>(
    trusted_state: &TS,
    untrusted_sh: &SH,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: &L,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
{
    // ensure the new height is higher
    let untrusted_height = untrusted_sh.header().height();
    let trusted_height = trusted_state.last_header().header().height();
    if untrusted_height <= trusted_height {
        // TODO: return err
    };

    // validate the untrusted header against its commit, vals, and next_vals
    let untrusted_header = untrusted_sh.header();
    let untrusted_commit = untrusted_sh.commit();
    validate_vals_and_commit(untrusted_header, untrusted_commit, untrusted_vals)?;
    validate_next_vals(untrusted_header, untrusted_next_vals)?;

    // if the new height is not sequential, check if we can skip
    if untrusted_height > trusted_height.increment() {
        let trusted_vals = trusted_state.validators();
        verify_commit_trusting(trusted_vals, untrusted_commit, trust_threshold)?;
    }

    // verify the untrusted commit
    verify_commit_full(untrusted_vals, untrusted_sh.commit())
}

/// Attempt to update the store to the given untrusted header.
/// Ensures our last trusted header hasn't expired yet, and that
/// the untrusted header can be verified using only our latest trusted
/// state from the store.
/// This function is primarily for use by IBC handlers.
pub fn verify_and_update_single<TS, SH, C, L, S>(
    untrusted_sh: &SH,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
    S: Store<TrustedState = TS>,
{
    // fetch the latest state and ensure it hasn't expired
    let trusted_state = store.get(Height::from(0))?;
    let trusted_sh = trusted_state.last_header();
    is_within_trust_period(trusted_sh.header(), trusting_period, now)?;

    verify_single(
        trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_threshold,
    )?;

    // the untrusted header is now trusted. update the store
    let new_trusted_state = TS::new(untrusted_sh, untrusted_next_vals);
    store.add(&new_trusted_state)
}

/// Attempt to update the store to the given untrusted height.
/// This function is recursive: it uses a bisection algorithm
/// to request data for intermediate heights as necessary.
/// Ensures our last trusted header hasn't expired yet, and that
/// data from the untrusted height can be verified, possibly using
/// data from intermediate heights.
/// This function is primarily for use by a light node.
pub fn verify_and_update_bisection<TS, SH, C, L, R, S>(
    untrusted_height: Height,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
    R: Requester<SignedHeader = SH, ValidatorSet = C::ValidatorSet>,
    S: Store<TrustedState = TS>,
{
    // Fetch the latest state and ensure it hasn't expired.
    // Note when calling itself recursively, this check is redundant,
    // but leaving it keeps things simple ...
    let trusted_state = store.get(Height::from(0))?;
    let trusted_sh = trusted_state.last_header();
    is_within_trust_period(trusted_sh.header(), trusting_period, now)?;

    // fetch the header and vals for the new height
    let untrusted_sh = &req.signed_header(untrusted_height)?;
    let untrusted_vals = &req.validator_set(untrusted_height)?;
    let untrusted_next_vals = &req.validator_set(untrusted_height.increment())?;

    // check if we can skip to this height and if it verifies.
    match verify_single(
        trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_threshold,
    ) {
        Ok(_) => {
            // Successfully verified!
            // Trust the new state and return.
            let new_trusted_state = TS::new(untrusted_sh, untrusted_next_vals);
            return store.add(&new_trusted_state);
        }
        Err(e) => {
            // If something went wrong, return the error.
            if e != Error::InsufficientVotingPower {
                return Err(e);
            }

            // Insufficient voting power to update.
            // Engage bisection, below.
        }
    }

    // Get the pivot height for bisection.
    let pivot_height: u64 = (trusted_sh.header().height().value() + untrusted_height.value()) / 2;

    // Recursive call to update to the pivot height.
    // When this completes, we will either return an error or
    // have updated the store to the pivot height.
    verify_and_update_bisection(
        Height::from(pivot_height),
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )?;

    // Recursive call to update to the original untrusted_height.
    verify_and_update_bisection(
        untrusted_height,
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )
}

mod tests {
    use super::*;
    use crate::{hash::Algorithm, Hash};
    use serde::Serialize;
    use sha2::{Digest, Sha256};

    #[derive(Clone, Debug, Serialize)]
    struct MockHeader {
        height: u64,
        time: SystemTime,
        vals: Hash,
        next_vals: Hash,
    }

    impl MockHeader {
        fn new(height: u64, time: SystemTime, vals: Hash, next_vals: Hash) -> MockHeader {
            MockHeader {
                height,
                time,
                vals,
                next_vals,
            }
        }
    }

    impl Header for MockHeader {
        type Time = SystemTime;

        fn height(&self) -> Height {
            Height::from(self.height)
        }
        fn bft_time(&self) -> Self::Time {
            self.time
        }
        fn validators_hash(&self) -> Hash {
            self.vals
        }
        fn next_validators_hash(&self) -> Hash {
            self.next_vals
        }
        fn hash(&self) -> Hash {
            json_hash(self)
        }
    }

    fn json_hash<T: ?Sized + Serialize>(value: &T) -> Hash {
        let encoded = serde_json::to_vec(value).unwrap();
        let hashed = Sha256::digest(&encoded);
        Hash::new(Algorithm::Sha256, &hashed).unwrap()
    }

    fn fixed_hash() -> Hash {
        Hash::new(Algorithm::Sha256, &Sha256::digest(&[5])).unwrap()
    }

    // vals are just ints, each has power 1
    #[derive(Clone, Debug, Serialize)]
    struct MockValSet {
        // NOTE: use HashSet instead?
        vals: Vec<usize>,
    }

    impl MockValSet {
        fn new(vals: Vec<usize>) -> MockValSet {
            MockValSet { vals }
        }
    }

    impl ValidatorSet for MockValSet {
        fn hash(&self) -> Hash {
            json_hash(&self)
        }
        fn total_power(&self) -> u64 {
            self.vals.len() as u64
        }
        fn len(&self) -> usize {
            self.vals.len()
        }
        fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }

    // just a list of vals that signed
    #[derive(Clone, Debug, Serialize)]
    struct MockCommit {
        hash: Hash,
        vals: Vec<usize>,
    }

    impl MockCommit {
        fn new(hash: Hash, vals: Vec<usize>) -> MockCommit {
            MockCommit { hash, vals }
        }
    }

    impl Commit for MockCommit {
        type ValidatorSet = MockValSet;

        fn header_hash(&self) -> Hash {
            self.hash
        }

        // just the intersection
        fn voting_power_in(&self, vals: &Self::ValidatorSet) -> Result<u64, Error> {
            let mut power = 0;
            for signer in self.vals.iter() {
                for val in vals.vals.iter() {
                    if signer == val {
                        power += 1
                    }
                }
            }
            Ok(power)
        }

        fn votes_len(&self) -> usize {
            self.vals.len()
        }
    }

    #[derive(Clone)]
    struct MockSignedHeader {
        header: MockHeader,
        commit: MockCommit,
    }

    impl MockSignedHeader {
        fn new(header: MockHeader, commit: MockCommit) -> Self {
            MockSignedHeader { header, commit }
        }
    }

    impl SignedHeader for MockSignedHeader {
        type Header = MockHeader;
        type Commit = MockCommit;
        fn header(&self) -> &Self::Header {
            &self.header
        }
        fn commit(&self) -> &Self::Commit {
            &self.commit
        }
    }

    // uses refs because the trait defines `new` to take refs ...
    struct MockState {
        header: MockSignedHeader,
        vals: MockValSet,
    }

    impl TrustedState for MockState{
        type LastHeader = MockSignedHeader;
        type ValidatorSet = MockValSet;

        // XXX: how to do this without cloning?!
        fn new(header: &Self::LastHeader, vals: &Self::ValidatorSet) -> MockState{
            MockState {
                header: header.clone(),
                vals: vals.clone(),
            }
        }
        fn last_header(&self) -> &Self::LastHeader {
            &self.header
        }
        fn validators(&self) -> &Self::ValidatorSet {
            &self.vals
        }
    }
    
    // XXX: Can we do without this mock since we have a default impl?
    struct MockThreshold {}
    impl TrustThreshold for MockThreshold{}

    #[test]
    fn test_verify_single() {
        let ts_time = SystemTime::UNIX_EPOCH;
        let ts_height = 1;
        let ts_vals = &MockValSet::new(vec!(0));
        let ts_header = MockHeader::new(ts_height, ts_time, ts_vals.hash(), ts_vals.hash());
        let ts_commit = MockCommit::new(ts_header.hash(), vec![0]);
        let ts_sh = &MockSignedHeader::new(ts_header, ts_commit);
        let ts = &MockState::new(ts_sh, ts_vals);

        let un_time = ts_time + Duration::new(10, 0);
        let un_height = 10;
        let un_vals = ts_vals;
        let un_next_vals = un_vals;
        let un_header = MockHeader::new(un_height, un_time, un_vals.hash(), un_vals.hash());
        let un_commit = MockCommit::new(un_header.hash(), vec![0]);
        let un_sh = &MockSignedHeader::new(un_header, un_commit);

        let threshold = &MockThreshold{};

        assert!(verify_single(ts, un_sh, un_vals, un_next_vals, threshold).is_ok());
    }
}
