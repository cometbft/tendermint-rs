//! Light client implementation as per the [Core Verification specification][1].
//!
//! [1]: https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification/verification.md

use contracts::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{fmt, time::Duration};

use crate::components::{clock::Clock, io::*, scheduler::*, verifier::*};
use crate::contracts::*;
use crate::{
    bail,
    errors::{Error, ErrorKind},
    state::State,
    types::{Height, LightBlock, PeerId, Status, TrustThreshold},
};

/// Verification parameters
///
/// TODO: Find a better name than `Options`
#[derive(Copy, Clone, Debug, PartialEq, Display, Serialize, Deserialize)]
#[display(fmt = "{:?}", self)]
pub struct Options {
    /// Defines what fraction of the total voting power of a known
    /// and trusted validator set is sufficient for a commit to be
    /// accepted going forward.
    pub trust_threshold: TrustThreshold,

    /// How long a validator set is trusted for (must be shorter than the chain's
    /// unbonding period)
    pub trusting_period: Duration,

    /// Correction parameter dealing with only approximately synchronized clocks.
    /// The local clock should always be ahead of timestamps from the blockchain; this
    /// is the maximum amount that the local clock may drift behind a timestamp from the
    /// blockchain.
    pub clock_drift: Duration,
}

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
pub struct LightClient {
    pub peer: PeerId,
    pub options: Options,
    clock: Box<dyn Clock>,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    io: Box<dyn Io>,
}

impl fmt::Debug for LightClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LightClient")
            .field("peer", &self.peer)
            .field("options", &self.options)
            .finish()
    }
}

impl LightClient {
    /// Constructs a new light client
    pub fn new(
        peer: PeerId,
        options: Options,
        clock: impl Clock + 'static,
        scheduler: impl Scheduler + 'static,
        verifier: impl Verifier + 'static,
        io: impl Io + 'static,
    ) -> Self {
        Self {
            peer,
            options,
            clock: Box::new(clock),
            scheduler: Box::new(scheduler),
            verifier: Box::new(verifier),
            io: Box::new(io),
        }
    }

    /// Attempt to update the light client to the highest block of the primary node.
    ///
    /// Note: This function delegates the actual work to `verify_to_target`.
    pub fn verify_to_highest(&mut self, state: &mut State) -> Result<LightBlock, Error> {
        let target_block = match self.io.fetch_light_block(self.peer, AtHeight::Highest) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        self.verify_to_target(target_block.height(), state)
    }

    /// Update the light client to a block of the primary node at the given height.
    ///
    /// This is the main function and uses the following components:
    ///
    /// - The I/O component is called to fetch the next light block.
    ///   It is the only component that communicates with other nodes.
    /// - The Verifier component checks whether a header is valid and checks if a new
    ///   light block should be trusted based on a previously verified light block.
    /// - The Scheduler component decides which height to try to verify next, in case
    ///   the current block pass verification but cannot be trusted yet.
    ///
    /// ## Implements
    /// - [LCV-DIST-SAFE.1]
    /// - [LCV-DIST-LIFE.1]
    /// - [LCV-PRE-TP.1]
    /// - [LCV-POST-LS.1]
    /// - [LCV-INV-TP.1]
    ///
    /// ## Precondition
    /// - The light store contains a light block within the trusting period [LCV-PRE-TP.1]
    ///
    /// ## Postcondition
    /// - The light store contains a light block that corresponds
    ///   to a block of the blockchain of height `target_height` [LCV-POST-LS.1]
    ///
    /// ## Error conditions
    /// - If the precondition is violated [LVC-PRE-TP.1]
    /// - If the core verification loop invariant is violated [LCV-INV-TP.1]
    /// - If verification of a light block fails
    /// - If it cannot fetch a block from the blockchain
    // #[pre(
    //     light_store_contains_block_within_trusting_period(
    //         state.light_store.as_ref(),
    //         self.options.trusting_period,
    //         self.clock.now(),
    //     )
    // )]
    #[post(
        ret.is_ok() ==> trusted_store_contains_block_at_target_height(
            state.light_store.as_ref(),
            target_height,
        )
    )]
    pub fn verify_to_target(
        &self,
        target_height: Height,
        state: &mut State,
    ) -> Result<LightBlock, Error> {
        // Let's first look in the store to see whether we have already successfully verified this block
        if let Some(light_block) = state.light_store.get_trusted_or_verified(target_height) {
            return Ok(light_block);
        }

        let mut current_height = target_height;

        loop {
            let now = self.clock.now();

            // Get the latest trusted state
            let trusted_state = state
                .light_store
                .latest_trusted_or_verified()
                .ok_or_else(|| ErrorKind::NoInitialTrustedState)?;

            if target_height < trusted_state.height() {
                bail!(ErrorKind::TargetLowerThanTrustedState {
                    target_height,
                    trusted_height: trusted_state.height()
                });
            }

            // Check invariant [LCV-INV-TP.1]
            if !is_within_trust_period(&trusted_state, self.options.trusting_period, now) {
                bail!(ErrorKind::TrustedStateOutsideTrustingPeriod {
                    trusted_state: Box::new(trusted_state),
                    options: self.options,
                });
            }

            // Log the current height as a dependency of the block at the target height
            state.trace_block(target_height, current_height);

            // If the trusted state is now at a height equal to the target height, we are done. [LCV-DIST-LIFE.1]
            if target_height == trusted_state.height() {
                return Ok(trusted_state);
            }

            // Fetch the block at the current height from the light store if already present,
            // or from the primary peer otherwise.
            let (current_block, status) = self.get_or_fetch_block(current_height, state)?;

            // Validate and verify the current block
            let verdict = self
                .verifier
                .verify(&current_block, &trusted_state, &self.options, now);

            match verdict {
                Verdict::Success => {
                    // Verification succeeded, add the block to the light store with
                    // the `Verified` status or higher if already trusted.
                    let new_status = Status::most_trusted(Status::Verified, status);
                    state.light_store.update(&current_block, new_status);
                }
                Verdict::Invalid(e) => {
                    // Verification failed, add the block to the light store with `Failed` status, and abort.
                    state.light_store.update(&current_block, Status::Failed);

                    bail!(ErrorKind::InvalidLightBlock(e))
                }
                Verdict::NotEnoughTrust(_) => {
                    // The current block cannot be trusted because of missing overlap in the validator sets.
                    // Add the block to the light store with `Unverified` status.
                    // This will engage bisection in an attempt to raise the height of the highest
                    // trusted state until there is enough overlap.
                    state.light_store.update(&current_block, Status::Unverified);
                }
            }

            // Compute the next height to fetch and verify
            current_height =
                self.scheduler
                    .schedule(state.light_store.as_ref(), current_height, target_height);
        }
    }

    /// Look in the light store for a block from the given peer at the given height,
    /// which has not previously failed verification (ie. its status is not `Failed`).
    ///
    /// If one cannot be found, fetch the block from the given peer and store
    /// it in the light store with `Unverified` status.
    ///
    /// ## Postcondition
    /// - The provider of block that is returned matches the given peer.
    #[post(ret.as_ref().map(|(lb, _)| lb.provider == self.peer).unwrap_or(true))]
    pub fn get_or_fetch_block(
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
            .fetch_light_block(self.peer, AtHeight::At(height))
            .map_err(ErrorKind::Io)?;

        state.light_store.insert(block.clone(), Status::Unverified);

        Ok((block, Status::Unverified))
    }
}
