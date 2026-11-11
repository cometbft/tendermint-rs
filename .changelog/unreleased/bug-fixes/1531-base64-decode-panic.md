- `[tendermint-proto]` Base64 decoding no longer panics on malformed padding such as
  `"="`, `"===="` or `"AAAA===="` (a panic inherited from `subtle-encoding`, also in
  release builds), and rejects input with data after padding (e.g. `"AB=C"`) instead of
  returning garbage. Such input now fails with an encoding error.
  ([\#1531](https://github.com/cometbft/tendermint-rs/pull/1531))
