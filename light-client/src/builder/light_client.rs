//! DSL for building a light client [`Instance`]

use tendermint::{block::Height, crypto::Sha256, merkle::MerkleHash, Hash};

#[cfg(feature = "rpc-client")]
use {
    crate::components::clock::SystemClock,
    crate::components::io::ProdIo,
    crate::components::scheduler,
    crate::verifier::{predicates::ProdPredicates, ProdVerifier},
    core::time::Duration,
    tendermint_rpc as rpc,
};

use crate::{
    builder::error::Error,
    components::{
        clock::Clock,
        io::{AtHeight, Io},
        scheduler::Scheduler,
    },
    instance::Instance,
    light_client::LightClient,
    state::{State, VerificationTrace},
    store::LightStore,
    verifier::{
        options::Options,
        predicates::VerificationPredicates,
        types::{LightBlock, PeerId, Status},
        Verifier,
    },
};

/// No trusted state has been set yet
pub struct NoTrustedState;

/// A trusted state has been set and validated
pub struct HasTrustedState;

/// Builder for a light client [`Instance`]
#[must_use]
pub struct LightClientBuilder<State, H: MerkleHash + Sha256 + Default> {
    peer_id: PeerId,
    options: Options,
    io: Box<dyn Io>,
    clock: Box<dyn Clock>,
    verifier: Box<dyn Verifier>,
    scheduler: Box<dyn Scheduler>,
    predicates: Box<dyn VerificationPredicates<Sha256 = H>>,
    light_store: Box<dyn LightStore>,

    #[allow(dead_code)]
    state: State,
}

impl<Current, H> LightClientBuilder<Current, H>
where
    H: MerkleHash + Sha256 + Default,
{
    /// Private method to move from one state to another
    fn with_state<Next>(self, state: Next) -> LightClientBuilder<Next, H> {
        LightClientBuilder {
            peer_id: self.peer_id,
            options: self.options,
            io: self.io,
            clock: self.clock,
            verifier: self.verifier,
            scheduler: self.scheduler,
            predicates: self.predicates,
            light_store: self.light_store,
            state,
        }
    }
}

#[cfg(feature = "rpc-client")]
impl LightClientBuilder<NoTrustedState, tendermint::crypto::default::Sha256> {
    /// Initialize a builder for a production (non-mock) light client.
    pub fn prod(
        peer_id: PeerId,
        rpc_client: rpc::HttpClient,
        light_store: Box<dyn LightStore>,
        options: Options,
        timeout: Option<Duration>,
    ) -> Self {
        #[allow(clippy::box_default)]
        Self::custom(
            peer_id,
            options,
            light_store,
            Box::new(ProdIo::new(peer_id, rpc_client, timeout)),
            Box::new(SystemClock),
            Box::new(ProdVerifier::default()),
            Box::new(scheduler::basic_bisecting_schedule),
            Box::new(ProdPredicates),
        )
    }
}

impl<H> LightClientBuilder<NoTrustedState, H>
where
    H: MerkleHash + Sha256 + Default,
{
    /// Initialize a builder for a custom light client, by providing all dependencies upfront.
    // TODO: redesign this, it's a builder API!
    #[allow(clippy::too_many_arguments)]
    pub fn custom(
        peer_id: PeerId,
        options: Options,
        light_store: Box<dyn LightStore>,
        io: Box<dyn Io>,
        clock: Box<dyn Clock>,
        verifier: Box<dyn Verifier>,
        scheduler: Box<dyn Scheduler>,
        predicates: Box<dyn VerificationPredicates<Sha256 = H>>,
    ) -> Self {
        Self {
            peer_id,
            io,
            verifier,
            light_store,
            clock,
            scheduler,
            options,
            predicates,
            state: NoTrustedState,
        }
    }

    /// Set the given light block as the initial trusted state.
    ///
    /// The block is validated before being inserted into the light store. If validation fails,
    /// the block is not added to the store and an error is returned.
    ///
    /// # Validation checks
    ///
    /// The block must pass the following checks:
    /// - The block header must be within the trusting period
    /// - The block header time must be in the past (accounting for clock drift)
    /// - The validator set must match the header's validators hash
    /// - The next validator set must match the header's next validators hash
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails. The light store remains unchanged in this case.
    pub fn trust_light_block(
        mut self,
        trusted_state: LightBlock,
    ) -> Result<LightClientBuilder<HasTrustedState, H>, Error> {
        self.validate(&trusted_state)?;
        self.light_store.insert(trusted_state, Status::Trusted);

        Ok(self.with_state(HasTrustedState))
    }

    /// Keep using the latest verified or trusted block in the light store.
    /// Such a block must exists otherwise this will fail.
    pub fn trust_from_store(self) -> Result<LightClientBuilder<HasTrustedState, H>, Error> {
        let trusted_state = self
            .light_store
            .highest_trusted_or_verified()
            .ok_or_else(Error::no_trusted_state_in_store)?;

        self.trust_light_block(trusted_state)
    }

    /// Set the block from the primary peer at the given height as the trusted state.
    pub fn trust_primary_at(
        self,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Result<LightClientBuilder<HasTrustedState, H>, Error> {
        let trusted_state = self
            .io
            .fetch_light_block(AtHeight::At(trusted_height))
            .map_err(Error::io)?;

        if trusted_state.height() != trusted_height {
            return Err(Error::height_mismatch(
                trusted_height,
                trusted_state.height(),
            ));
        }

        let header_hash = trusted_state.signed_header.header.hash_with::<H>();

        if header_hash != trusted_hash {
            return Err(Error::hash_mismatch(trusted_hash, header_hash));
        }

        self.trust_light_block(trusted_state)
    }

    /// Validates a light block before it can be trusted.
    ///
    /// Performs the following checks:
    /// - Verifies the block header is within the trusting period
    /// - Verifies the block header time is in the past (accounting for clock drift)
    /// - Verifies the validator set matches the header's validators hash
    /// - Verifies the next validator set matches the header's next validators hash
    ///
    /// # Errors
    ///
    /// Returns an error if any validation check fails.
    fn validate(&self, light_block: &LightBlock) -> Result<(), Error> {
        let header = &light_block.signed_header.header;
        let now = self.clock.now();

        self.predicates
            .is_within_trust_period(header.time, self.options.trusting_period, now)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .is_header_from_past(header.time, self.options.clock_drift, now)
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .validator_sets_match(
                &light_block.validators,
                light_block.signed_header.header.validators_hash,
            )
            .map_err(Error::invalid_light_block)?;

        self.predicates
            .next_validators_match(
                &light_block.next_validators,
                light_block.signed_header.header.next_validators_hash,
            )
            .map_err(Error::invalid_light_block)?;

        Ok(())
    }
}

impl<H> LightClientBuilder<HasTrustedState, H>
where
    H: MerkleHash + Sha256 + Default,
{
    /// Build the light client [`Instance`].
    #[must_use]
    pub fn build(self) -> Instance {
        let state = State {
            light_store: self.light_store,
            verification_trace: VerificationTrace::new(),
        };

        let light_client = LightClient::from_boxed(
            self.peer_id,
            self.options,
            self.clock,
            self.scheduler,
            self.verifier,
            self.io,
        );

        Instance::new(light_client, state)
    }
}
