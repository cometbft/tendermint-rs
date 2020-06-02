use contracts::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::components::{io::*, scheduler::*, verifier::*};
use crate::contracts::*;
use crate::prelude::*;

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
    /// The trusting period
    pub trusting_period: Duration,
    /// Correction parameter dealing with only approximately synchronized clocks.
    pub clock_drift: Duration,
    /// The current time
    pub now: Time,
}

impl Options {
    /// Override the stored current time with the given one.
    pub fn with_now(self, now: Time) -> Self {
        Self { now, ..self }
    }
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
    state: State,
    options: Options,
    clock: Box<dyn Clock>,
    scheduler: Box<dyn Scheduler>,
    verifier: Box<dyn Verifier>,
    fork_detector: Box<dyn ForkDetector>,
    io: Box<dyn Io>,
}

impl LightClient {
    /// Constructs a new light client
    pub fn new(
        state: State,
        options: Options,
        clock: impl Clock + 'static,
        scheduler: impl Scheduler + 'static,
        verifier: impl Verifier + 'static,
        fork_detector: impl ForkDetector + 'static,
        io: impl Io + 'static,
    ) -> Self {
        Self {
            state,
            options,
            clock: Box::new(clock),
            scheduler: Box::new(scheduler),
            verifier: Box::new(verifier),
            fork_detector: Box::new(fork_detector),
            io: Box::new(io),
        }
    }

    /// Attempt to update the light client to the latest block of the primary node.
    ///
    /// Note: This functin delegates the actual work to `verify_to_target`.
    pub fn verify_to_highest(&mut self) -> Result<LightBlock, Error> {
        let peer = self.state.peers.primary;
        let target_block = match self.io.fetch_light_block(peer, LATEST_HEIGHT) {
            Ok(last_block) => last_block,
            Err(io_error) => bail!(ErrorKind::Io(io_error)),
        };

        self.verify_to_target(target_block.height())
    }

    /// Attemps to update the light client to a block of the primary node at the given height.
    ///
    /// This is the main function and uses the following components:
    ///
    /// - The I/O component is called to download the next light block.
    ///   It is the only component that communicates with other nodes.
    /// - The Verifier component checks whether a header is valid and checks if a new
    ///   light block should be trusted based on a previously verified light block.
    /// - The Scheduler component decides which height to try to verify next.
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
    //         self.state.light_store.as_ref(),
    //         self.options.trusting_period,
    //         self.clock.now(),
    //     )
    // )]
    #[post(
        ret.is_ok() ==> trusted_store_contains_block_at_target_height(
            self.state.light_store.as_ref(),
            target_height,
        )
    )]
    pub fn verify_to_target(&mut self, target_height: Height) -> Result<LightBlock, Error> {
        // Override the `now` fields in the given verification options with the current time,
        // as per the given `clock`.
        let options = self.options.with_now(self.clock.now());

        let mut current_height = target_height;

        loop {
            // Get the latest trusted state
            let trusted_state = self
                .state
                .light_store
                .latest(VerifiedStatus::Verified)
                .ok_or_else(|| ErrorKind::NoInitialTrustedState)?;

            // Check invariant [LCV-INV-TP.1]
            if !is_within_trust_period(&trusted_state, options.trusting_period, options.now) {
                bail!(ErrorKind::TrustedStateOutsideTrustingPeriod {
                    trusted_state: Box::new(trusted_state),
                    options,
                });
            }

            // Trace the current height as a dependency of the block at the target height
            self.state.trace_block(target_height, current_height);

            // If the trusted state is now at the height greater or equal to the target height,
            // we now trust this target height, and are thus done :) [LCV-DIST-LIFE.1]
            if target_height <= trusted_state.height() {
                return Ok(trusted_state);
            }

            // Fetch the block at the current height from the primary node
            let current_block =
                self.get_or_fetch_block(self.state.peers.primary, current_height)?;

            // Validate and verify the current block
            let verdict = self
                .verifier
                .verify(&current_block, &trusted_state, &options);

            match verdict {
                Verdict::Success => {
                    // Verification succeeded, add the block to the light store with `verified` status
                    self.state
                        .light_store
                        .update(current_block, VerifiedStatus::Verified);
                }
                Verdict::Invalid(e) => {
                    // Verification failed, add the block to the light store with `failed` status, and abort.
                    self.state
                        .light_store
                        .update(current_block, VerifiedStatus::Failed);

                    bail!(ErrorKind::InvalidLightBlock(e))
                }
                Verdict::NotEnoughTrust(_) => {
                    // The current block cannot be trusted because of missing overlap in the validator sets.
                    // Add the block to the light store with `unverified` status.
                    // This will engage bisection in an attempt to raise the height of the latest
                    // trusted state until there is enough overlap.
                    self.state
                        .light_store
                        .update(current_block, VerifiedStatus::Unverified);
                }
            }

            // Compute the next height to fetch and verify
            current_height = self.scheduler.schedule(
                self.state.light_store.as_ref(),
                current_height,
                target_height,
            );
        }
    }

    /// TODO
    pub fn detect_forks(&self) -> Result<(), Error> {
        let light_blocks = self
            .state
            .light_store
            .all(VerifiedStatus::Verified)
            .collect();

        let result = self.fork_detector.detect(light_blocks);

        match result {
            ForkDetection::NotDetected => (),    // TODO
            ForkDetection::Detected(_, _) => (), // TODO
        }

        Ok(())
    }

    /// Get the verification trace for the block at target_height.
    pub fn get_trace(&self, target_height: Height) -> Vec<LightBlock> {
        self.state.get_trace(target_height)
    }

    /// Look in the light store for a block from the given peer at the given height.
    /// If one cannot be found, fetch the block from the given peer.
    ///
    /// ## Postcondition
    /// - The provider of block that is returned matches the given peer.
    // TODO: Uncomment when provider field is available
    // #[post(ret.map(|lb| lb.provider == peer).unwrap_or(false))]
    fn get_or_fetch_block(
        &mut self,
        peer: PeerId,
        current_height: Height,
    ) -> Result<LightBlock, Error> {
        let current_block = self
            .state
            .light_store
            .get(current_height, VerifiedStatus::Verified)
            // .filter(|lb| lb.provider == peer)
            .or_else(|| {
                self.state
                    .light_store
                    .get(current_height, VerifiedStatus::Unverified)
                // .filter(|lb| lb.provider == peer)
            });

        if let Some(current_block) = current_block {
            return Ok(current_block);
        }

        self.io
            .fetch_light_block(peer, current_height)
            .map(|current_block| {
                self.state
                    .light_store
                    .insert(current_block.clone(), VerifiedStatus::Unverified);

                current_block
            })
            .map_err(|e| ErrorKind::Io(e).into())
    }
}
