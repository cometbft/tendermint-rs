- `[tendermint]` `public_key::Algorithm` gains an `MlDsa65` variant
  (`"ml_dsa_65"`), so consensus params from ML-DSA chains parse instead of
  failing (JSON) or being read as Ed25519 (protobuf). `Signature::new` now
  also accepts 3309-byte (ML-DSA-65) signatures, independent of the
  `mldsa65` feature
  ([\#1533](https://github.com/cometbft/tendermint-rs/pull/1533))
