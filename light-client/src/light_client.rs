//! Light client implementation as per the [Core Verification specification][1].
//!
//! [1]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification/verification.md

use sp_std::fmt;

use sp_std::marker::PhantomData;
use tendermint_light_client_verifier::host_functions::CryptoProvider;

// Re-export for backward compatibility
pub use crate::verifier::options::Options;
use crate::{
    components::{clock::Clock, io::*, scheduler::*},
    contracts::*,
    errors::Error,
    state::State,
    verifier::{
        types::{Height, LightBlock, PeerId, Status},
        Verdict, Verifier,
    },
};

/// The light client implements a read operation of a header from the blockchain,
/// by communicating with full nodes. As full nodes may be faulty, it cannot trust
/// the received information, but the light client has to check whether the header
/// it receives coincides with the one generated by Tendermint consensus.
///
/// In the Tendermint blockchain, the validator set may change with every new block.
/// The staking and unbonding mechanism induces a security model: starting at time
/// of the header, more than two-thirds of the next validators of a new block are
/// correct for the duration of the trusted period.  The fault-tolerant read operation
/// is designed for this security model.
pub struct LightClient<HostFunctions> {
    /// The peer id of the peer this client is connected to
    pub peer: PeerId,
    /// Options for this light client
    pub options: Options,

    clock: Box<dyn Clock>,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    io: Box<dyn AsyncIo>,
    _phantom: PhantomData<HostFunctions>,
}

impl<HostFunctions> fmt::Debug for LightClient<HostFunctions> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LightClient")
            .field("peer", &self.peer)
            .field("options", &self.options)
            .finish()
    }
}

impl<HostFunctions> LightClient<HostFunctions>
    where
        HostFunctions: CryptoProvider,
{
    /// Constructs a new light client
    pub fn new(
        peer: PeerId,
        options: Options,
        clock: impl Clock + 'static,
        scheduler: impl Scheduler + 'static,
        verifier: impl Verifier + 'static,
        io: impl AsyncIo + 'static,
    ) -> Self {
        Self {
            peer,
            options,
            clock: Box::new(clock),
            scheduler: Box::new(scheduler),
            verifier: Box::new(verifier),
            io: Box::new(io),
            _phantom: PhantomData,
        }
    }

    /// Constructs a new light client from boxed components
    pub fn from_boxed(
        peer: PeerId,
        options: Options,
        clock: Box<dyn Clock>,
        scheduler: Box<dyn Scheduler>,
        verifier: Box<dyn Verifier>,
        io: Box<dyn AsyncIo>,
    ) -> Self {
        Self {
            peer,
            options,
            clock,
            scheduler,
            verifier,
            io,
            _phantom: PhantomData,
        }
    }

    /// Attempt to update the light client to the highest block of the primary node.
    ///
    /// Note: This function delegates the actual work to `verify_to_target`.
    pub async fn verify_to_highest(&mut self, state: &mut State) -> Result<LightBlock, Error> {
        let target_block = self
            .io
            .fetch_light_block(AtHeight::Highest)
            .await
            .map_err(Error::io)?;

        self.verify_to_target(target_block.height(), state).await
    }

    /// Update the light client to a block of the primary node at the given height.
    ///
    /// This is the main function and uses the following components:
    ///
    /// - The I/O component is called to fetch the next light block. It is the only component that
    ///   communicates with other nodes.
    /// - The Verifier component checks whether a header is valid and checks if a new light block
    ///   should be trusted based on a previously verified light block.
    /// - When doing _forward_ verification, the Scheduler component decides which height to try to
    ///   verify next, in case the current block pass verification but cannot be trusted yet.
    /// - When doing _backward_ verification, the Hasher component is used to determine whether the
    ///   `last_block_id` hash of a block matches the hash of the block right below it.
    ///
    /// ## Implements
    /// - [LCV-DIST-SAFE.1]
    /// - [LCV-DIST-LIFE.1]
    /// - [LCV-PRE-TP.1]
    /// - [LCV-POST-LS.1]
    /// - [LCV-INV-TP.1]
    ///
    /// ## Postcondition
    /// - The light store contains a light block that corresponds to a block of the blockchain of
    ///   height `target_height` [LCV-POST-LS.1]
    ///
    /// ## Error conditions
    /// - The light store does not contains a trusted light block within the trusting period
    ///   [LCV-PRE-TP.1]
    /// - If the core verification loop invariant is violated [LCV-INV-TP.1]
    /// - If verification of a light block fails
    /// - If the fetching a light block from the primary node fails
    #[allow(clippy::nonminimal_bool)]
    pub async fn verify_to_target(
        &self,
        target_height: Height,
        state: &mut State,
    ) -> Result<LightBlock, Error> {
        // Let's first look in the store to see whether
        // we have already successfully verified this block.
        if let Some(light_block) = state.light_store.get_trusted_or_verified(target_height) {
            return Ok(light_block);
        }

        // Get the highest trusted state
        let highest = state
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(Error::no_initial_trusted_state)?;

        let block = if target_height >= highest.height() {
            // Perform forward verification with bisection
            self.verify_forward(target_height, state).await?
        } else {
            // Perform sequential backward verification
            self.verify_backward(target_height, state).await?
        };

        assert!(trusted_store_contains_block_at_target_height(
            state.light_store.as_ref(),
            target_height,
        ));

        Ok(block)
    }

    /// Perform forward verification with bisection.
    async fn verify_forward(
        &self,
        target_height: Height,
        state: &mut State,
    ) -> Result<LightBlock, Error> {
        let mut current_height = target_height;

        loop {
            let now = self.clock.now();

            // Get the latest trusted state
            let trusted_block = state
                .light_store
                .highest_trusted_or_verified()
                .ok_or_else(Error::no_initial_trusted_state)?;

            if target_height < trusted_block.height() {
                return Err(Error::target_lower_than_trusted_state(
                    target_height,
                    trusted_block.height(),
                ));
            }

            // Check invariant [LCV-INV-TP.1]
            if !is_within_trust_period(&trusted_block, self.options.trusting_period, now) {
                return Err(Error::trusted_state_outside_trusting_period(
                    Box::new(trusted_block),
                    self.options,
                ));
            }

            // Log the current height as a dependency of the block at the target height
            state.trace_block(target_height, current_height);

            // If the trusted state is now at a height equal to the target height, we are done.
            // [LCV-DIST-LIFE.1]
            if target_height == trusted_block.height() {
                return Ok(trusted_block);
            }

            // Fetch the block at the current height from the light store if already present,
            // or from the primary peer otherwise.
            let (current_block, status) = self.get_or_fetch_block(current_height, state).await?;

            // Validate and verify the current block
            let verdict = self.verifier.verify(
                current_block.as_untrusted_state(),
                trusted_block.as_trusted_state(),
                &self.options,
                now,
            );

            match verdict {
                Verdict::Success => {
                    // Verification succeeded, add the block to the light store with
                    // the `Verified` status or higher if already trusted.
                    let new_status = Status::most_trusted(Status::Verified, status);
                    state.light_store.update(&current_block, new_status);
                }
                Verdict::Invalid(e) => {
                    // Verification failed, add the block to the light store with `Failed` status,
                    // and abort.
                    state.light_store.update(&current_block, Status::Failed);

                    return Err(Error::invalid_light_block(e));
                }
                Verdict::NotEnoughTrust(_) => {
                    // The current block cannot be trusted because of a missing overlap in the
                    // validator sets. Add the block to the light store with
                    // the `Unverified` status. This will engage bisection in an
                    // attempt to raise the height of the highest trusted state
                    // until there is enough overlap.
                    state.light_store.update(&current_block, Status::Unverified);
                }
            }

            // Compute the next height to fetch and verify
            current_height =
                self.scheduler
                    .schedule(state.light_store.as_ref(), current_height, target_height);
        }
    }

    /// Stub for when "unstable" feature is disabled.
    #[doc(hidden)]
    #[cfg(not(feature = "unstable"))]
    async fn verify_backward(
        &self,
        target_height: Height,
        state: &mut State,
    ) -> Result<LightBlock, Error> {
        let trusted_state = state
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(Error::no_initial_trusted_state)?;

        Err(Error::target_lower_than_trusted_state(
            target_height,
            trusted_state.height(),
        ))
    }

    /// Perform sequential backward verification.
    ///
    /// Backward verification is implemented by taking a sliding window
    /// of length two between the trusted state and the target block and
    /// checking whether the last_block_id hash of the higher block
    /// matches the computed hash of the lower block.
    ///
    /// ## Performance
    /// The algorithm implemented is very inefficient in case the target
    /// block is much lower than the highest trusted state.
    /// For a trusted state at height `T`, and a target block at height `H`,
    /// it will fetch and check hashes of `T - H` blocks.
    ///
    /// ## Stability
    /// This feature is only available if the `unstable` flag of is enabled.
    /// If the flag is disabled, then any attempt to verify a block whose
    /// height is lower than the highest trusted state will result in a
    /// `TargetLowerThanTrustedState` error.
    #[cfg(feature = "unstable")]
    async fn verify_backward(
        &self,
        target_height: Height,
        state: &mut State,
    ) -> Result<LightBlock, Error> {
        use sp_std::convert::TryFrom;
        use tendermint::Hash;

        use tendermint_light_client_verifier::merkle::simple_hash_from_byte_vectors;

        let root = state
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(Error::no_initial_trusted_state)?;

        assert!(root.height() >= target_height);

        // Check invariant [LCV-INV-TP.1]
        if !is_within_trust_period(&root, self.options.trusting_period, self.clock.now()) {
            return Err(Error::trusted_state_outside_trusting_period(
                Box::new(root),
                self.options,
            ));
        }

        // Compute a range of `Height`s from `trusted_height - 1` to `target_height`, inclusive.
        let range = (target_height.value()..root.height().value()).rev();
        let heights = range.map(|h| Height::try_from(h).unwrap());

        let mut latest = root;

        for height in heights {
            let (current, _status) = self.get_or_fetch_block(height, state).await?;

            let latest_last_block_id = latest
                .signed_header
                .header
                .last_block_id
                .ok_or_else(|| Error::missing_last_block_id(latest.height()))?;

            let current_hash = {
                let serialized = current.signed_header.header.serialize_to_preimage();
                Hash::Sha256(simple_hash_from_byte_vectors::<HostFunctions>(serialized))
            };

            if current_hash != latest_last_block_id.hash {
                return Err(Error::invalid_adjacent_headers(
                    current_hash,
                    latest_last_block_id.hash,
                ));
            }

            // `latest` and `current` are linked together by `last_block_id`,
            // therefore it is not relevant which we verified first.
            // For consistency, we say that `latest` was verifed using
            // `current` so that the trace is always pointing down the chain.
            state.light_store.insert(current.clone(), Status::Trusted);
            state.light_store.insert(latest.clone(), Status::Trusted);
            state.trace_block(latest.height(), current.height());

            latest = current;
        }

        // We reached the target height.
        assert_eq!(latest.height(), target_height);

        Ok(latest)
    }

    /// Look in the light store for a block from the given peer at the given height,
    /// which has not previously failed verification (ie. its status is not `Failed`).
    ///
    /// If one cannot be found, fetch the block from the given peer and store
    /// it in the light store with `Unverified` status.
    ///
    /// ## Postcondition
    /// - The provider of block that is returned matches the given peer.
    pub async fn get_or_fetch_block(
        &self,
        height: Height,
        state: &mut State,
    ) -> Result<(LightBlock, Status), Error> {
        let block = state.light_store.get_non_failed(height);

        if let Some(block) = block {
            return Ok(block);
        }

        let block = self
            .io
            .fetch_light_block(AtHeight::At(height))
            .await
            .map_err(Error::io)?;

        assert!(block.provider == self.peer);

        state.light_store.insert(block.clone(), Status::Unverified);

        Ok((block, Status::Unverified))
    }
}
