- `[tendermint]` Implement `PartialEq` between `block::Height` and the unsigned integer types `u8`, `u16`, `u32`, and `u64` (both directions). Allows `height == 5_u64` and `5_u64 == height` directly, without going through `.value()`.
  ([\#1507](https://github.com/informalsystems/tendermint-rs/issues/1507))
