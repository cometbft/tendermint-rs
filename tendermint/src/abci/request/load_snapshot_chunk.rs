use crate::{block, prelude::*};

#[doc = include_str!("../doc/request-loadsnapshotchunk.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoadSnapshotChunk {
    /// The height of the snapshot the chunks belong to.
    pub height: block::Height,
    /// An application-specific identifier of the format of the snapshot chunk.
    pub format: u32,
    /// The chunk index, starting from `0` for the initial chunk.
    pub chunk: u32,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<LoadSnapshotChunk> for pb::RequestLoadSnapshotChunk {
    fn from(load_snapshot_chunk: LoadSnapshotChunk) -> Self {
        Self {
            height: load_snapshot_chunk.height.into(),
            format: load_snapshot_chunk.format,
            chunk: load_snapshot_chunk.chunk,
        }
    }
}

impl TryFrom<pb::RequestLoadSnapshotChunk> for LoadSnapshotChunk {
    type Error = crate::Error;

    fn try_from(load_snapshot_chunk: pb::RequestLoadSnapshotChunk) -> Result<Self, Self::Error> {
        Ok(Self {
            height: load_snapshot_chunk.height.try_into()?,
            format: load_snapshot_chunk.format,
            chunk: load_snapshot_chunk.chunk,
        })
    }
}

impl Protobuf<pb::RequestLoadSnapshotChunk> for LoadSnapshotChunk {}
