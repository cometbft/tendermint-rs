use tendermint::{crypto::default::Sha256, evidence::Evidence};
use tendermint_light_client_verifier::types::Status;
use tracing::{debug, error, error_span, info, warn};

use crate::{
    state::State,
    store::{memory::MemoryStore, LightStore},
    verifier::types::LightBlock,
};

use super::{
    error::detector::Error as DetectorError, evidence::make_evidence, provider::Provider,
    trace::Trace,
};

// FIXME: Allow the hasher to be configured
type Hasher = Sha256;

#[derive(Debug)]
pub struct ReportedEvidence {
    pub against_primary: Evidence, // FIXME: Change this to LightClientAttackEvidence
    pub against_witness: Evidence, // FIXME: Change this to LightClientAttackEvidence
}

/// Handles the primary style of attack, which is where a primary and witness have
/// two headers of the same height but with different hashes
pub async fn handle_conflicting_headers(
    primary: &Provider,
    witness: &Provider,
    primary_trace: &Trace,
    challenging_block: &LightBlock,
) -> Result<ReportedEvidence, DetectorError> {
    let _span =
        error_span!("handle_conflicting_headers", primary = %primary.peer_id(), witness = %witness.peer_id())
            .entered();

    let (witness_trace, primary_block) =
        examine_conflicting_header_against_trace(primary_trace, challenging_block, witness)
            .map_err(|e| {
                error!("Error validating witness's divergent header: {e}");

                e // FIXME: Return special error
            })?;

    // We are suspecting that the primary is faulty, hence we hold the witness as the source of truth
    // and generate evidence against the primary that we can send to the witness

    let common_block = witness_trace.first();
    let trusted_block = witness_trace.last();

    let evidence_against_primary = Evidence::from(make_evidence(
        primary_block.clone(),
        trusted_block.clone(),
        common_block.clone(),
    ));

    warn!("ATTEMPTED ATTACK DETECTED. Sending evidence against primary by witness...");

    debug!(
        "Evidence against primary: {}",
        serde_json::to_string_pretty(&evidence_against_primary).unwrap()
    );

    let result = witness
        .report_evidence(evidence_against_primary.clone())
        .await;

    match result {
        Ok(hash) => {
            info!("Successfully submitted evidence against primary. Evidence hash: {hash}",);
        },
        Err(e) => {
            error!("Failed to submit evidence against primary: {e}");
        },
    }

    if primary_block.signed_header.commit.round != trusted_block.signed_header.commit.round {
        error!(
            "The light client has detected, and prevented, an attempted amnesia attack.
            We think this attack is pretty unlikely, so if you see it, that's interesting to us.
            Can you let us know by opening an issue through https://github.com/tendermint/tendermint/issues/new"
        );
    }

    // This may not be valid because the witness itself is at fault. So now we reverse it, examining the
    // trace provided by the witness and holding the primary as the source of truth. Note: primary may not
    // respond but this is okay as we will halt anyway.
    let (primary_trace, witness_block) =
        examine_conflicting_header_against_trace(&witness_trace, &primary_block, primary).map_err(
            |e| {
                error!("Error validating primary's divergent header: {e}");

                e // FIXME: Return special error
            },
        )?;

    // We now use the primary trace to create evidence against the witness and send it to the primary
    let common_block = primary_trace.first();
    let trusted_block = primary_trace.last();

    let evidence_against_witness = Evidence::from(make_evidence(
        witness_block,
        trusted_block.clone(),
        common_block.clone(),
    ));

    warn!("Sending evidence against witness by primary...");

    debug!(
        "Evidence against witness: {}",
        serde_json::to_string_pretty(&evidence_against_witness).unwrap()
    );

    let result = primary
        .report_evidence(evidence_against_witness.clone())
        .await;

    match result {
        Ok(hash) => {
            info!("Successfully submitted evidence against witness. Evidence hash: {hash}",);
        },
        Err(e) => {
            error!("Failed to submit evidence against witness: {e}");
        },
    }

    Ok(ReportedEvidence {
        against_primary: evidence_against_primary,
        against_witness: evidence_against_witness,
    })
}

#[derive(Debug)]
enum ExaminationResult {
    Continue(LightBlock),
    Divergence(Trace, LightBlock),
}

fn examine_conflicting_header_against_trace_block(
    source: &Provider,
    trace_block: &LightBlock,
    target_block: &LightBlock,
    prev_verified_block: Option<LightBlock>,
) -> Result<ExaminationResult, DetectorError> {
    // This case only happens in a forward lunatic attack. We treat the block with the
    // height directly after the targetBlock as the divergent block
    if trace_block.height() > target_block.height() {
        // sanity check that the time of the traceBlock is indeed less than that of the targetBlock. If the trace
        // was correctly verified we should expect monotonically increasing time. This means that if the block at
        // the end of the trace has a lesser time than the target block then all blocks in the trace should have a
        // lesser time
        if trace_block.time() > target_block.time() {
            return Err(DetectorError::trace_block_after_target_block(
                trace_block.time(),
                target_block.time(),
            ));
        }

        // Before sending back the divergent block and trace we need to ensure we have verified
        // the final gap between the previouslyVerifiedBlock and the targetBlock
        if let Some(prev_verified_block) = &prev_verified_block {
            if prev_verified_block.height() != target_block.height() {
                let source_trace =
                    verify_skipping(source, prev_verified_block.clone(), target_block.clone())?;

                return Ok(ExaminationResult::Divergence(
                    source_trace,
                    trace_block.clone(),
                ));
            }
        }
    }

    // get the corresponding block from the source to verify and match up against the traceBlock
    let source_block = if trace_block.height() == target_block.height() {
        target_block.clone()
    } else {
        source
            .fetch_light_block(trace_block.height())
            .map_err(DetectorError::light_client)?
    };

    let source_block_hash = source_block.signed_header.header.hash_with::<Hasher>();
    let trace_block_hash = trace_block.signed_header.header.hash_with::<Hasher>();

    match prev_verified_block {
        None => {
            // the first block in the trace MUST be the same to the light block that the source produces
            // else we cannot continue with verification.
            if source_block_hash != trace_block_hash {
                Err(
                    DetectorError::trusted_hash_different_from_source_first_block(
                        source_block_hash,
                        trace_block_hash,
                    ),
                )
            } else {
                Ok(ExaminationResult::Continue(source_block))
            }
        },
        Some(prev_verified_block) => {
            // we check that the source provider can verify a block at the same height of the
            // intermediate height
            let source_trace = verify_skipping(source, prev_verified_block, source_block.clone())?;

            // check if the headers verified by the source has diverged from the trace
            if source_block_hash != trace_block_hash {
                // Bifurcation point found!
                return Ok(ExaminationResult::Divergence(
                    source_trace,
                    trace_block.clone(),
                ));
            }

            // headers are still the same, continue
            Ok(ExaminationResult::Continue(source_block))
        },
    }
}

fn examine_conflicting_header_against_trace(
    trace: &Trace,
    target_block: &LightBlock,
    source: &Provider,
) -> Result<(Trace, LightBlock), DetectorError> {
    let trusted_block = trace.first();

    if target_block.height() < trusted_block.height() {
        return Err(DetectorError::target_block_lower_than_trusted(
            target_block.height(),
            trusted_block.height(),
        ));
    }

    let mut previously_verified_block: Option<LightBlock> = None;

    for trace_block in trace.iter() {
        let result = examine_conflicting_header_against_trace_block(
            source,
            trace_block,
            target_block,
            previously_verified_block,
        )?;

        match result {
            ExaminationResult::Continue(prev_verified_block) => {
                previously_verified_block = Some(prev_verified_block);
                continue;
            },
            ExaminationResult::Divergence(source_trace, trace_block) => {
                return Ok((source_trace, trace_block));
            },
        }
    }

    // We have reached the end of the trace. This should never happen. This can only happen if one of the stated
    // prerequisites to this function were not met.
    // Namely that either trace[len(trace)-1].Height < targetBlock.Height
    // or that trace[i].Hash() != targetBlock.Hash()
    Err(DetectorError::no_divergence())
}

fn verify_skipping(
    source: &Provider,
    trusted: LightBlock,
    target: LightBlock,
) -> Result<Trace, DetectorError> {
    let target_height = target.height();

    let mut store = MemoryStore::new();
    store.insert(trusted, Status::Trusted);
    store.insert(target, Status::Unverified);

    let mut state = State::new(store);

    let _ = source
        .verify_to_height_with_state(target_height, &mut state)
        .map_err(DetectorError::light_client)?;

    let blocks = state.get_trace(target_height);
    let source_trace = Trace::new(blocks).map_err(|e| DetectorError::trace_too_short(e.trace))?;

    Ok(source_trace)
}
