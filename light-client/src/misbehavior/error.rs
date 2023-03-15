use flex_error::define_error;
use tendermint::{block::Height, Hash, Time};
use tendermint_light_client_verifier::types::LightBlock;

use crate::{components::io::IoError, errors::Error as LightClientError};

pub mod divergence {
    use crate::misbehavior::conflict::ReportedEvidence;

    use super::*;

    flex_error::define_error! {
        #[derive(Debug)]
        Error {
            NoWitnesses
                |_| { format_args!("No witnesses provided") },

            Divergence
                { reported_evidence: ReportedEvidence }
                |e| { format_args!("Divergence detected, found evidence: {:#?}", e.reported_evidence) },

            FailedHeaderCrossReferencing
                |_| { format_args!("Failed to cross-reference header with witness") },

            TraceTooShort
                { trace: Vec<LightBlock> }
                |e| { format_args!("Trace too short, length: {}", e.trace.len()) },
        }
    }
}

pub mod detector {
    use super::*;

    define_error! {
        #[derive(Debug)]
        Error {
            Io
                [ IoError ]
                |_| { "I/O error" },

            LightClient
                [ LightClientError ]
                |_| { "light client error" },

            TargetBlockLowerThanTrusted
                {
                    target_height: Height,
                    trusted_height: Height,
                }
                |e| {
                    format_args!(
                        "target block height ({}) lower than trusted block height ({})",
                        e.target_height, e.trusted_height
                    )
                },

            TrustedHashDifferentFromSourceFirstBlock
                {
                    expected_hash: Hash,
                    got_hash: Hash,
                }
                |e| {
                    format_args!(
                        "trusted block is different to the source's first block. Expected hash: {}, got: {}",
                        e.expected_hash, e.got_hash
                    )
                },

            TraceTooShort
                {
                    trace: Vec<LightBlock>,
                }
                |e| {
                    format_args!(
                        "trace is too short. Expected at least 2 blocks, got {} block",
                        e.trace.len()
                    )
                },

            TraceBlockAfterTargetBlock
                {
                    trace_time: Time,
                    target_time: Time,
                }
                |e| {
                    format_args!(
                        "trace block ({}) is after target block ({})",
                        e.trace_time, e.target_time
                    )
                },

            NoWitnesses
                |_| { "no witnesses provided" },

            NoDivergence
                |_| { "expected divergence between conflicting headers but none found" },
        }
    }
}
