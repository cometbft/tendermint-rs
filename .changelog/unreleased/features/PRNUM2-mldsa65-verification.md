- `[tendermint]` Add ML-DSA-65 (FIPS 204) validator key support behind the new
  `mldsa65` feature (requires Rust 1.87+), matching CometBFT v0.40 and the
  verification-only backport in v0.38.26: `PublicKey::MlDsa65` (JSON type
  `cometbft/PubKeyMlDsa65`, protobuf `PublicKey.mldsa65`), addresses as
  `SHA256(pubkey)[:20]`, and pure ML-DSA signature verification (empty
  context) in the default verifier. Signing is not supported. Only the v0.38
  protobuf types can carry an ML-DSA-65 key; converting one into the v0.34 or
  v0.37 `PublicKey` yields an empty `sum`, which fails to decode.
  `PublicKey::to_bech32` uses the amino prefix derived from the registered
  name `cometbft/PubKeyMlDsa65`; CometBFT itself defines no bech32 form.
  ([\#PRNUM2](https://github.com/cometbft/tendermint-rs/pull/PRNUM2))
- `[tendermint-light-client-verifier]` Add an `mldsa65` feature that enables
  verification of commits signed by ML-DSA-65 validators
  ([\#PRNUM2](https://github.com/cometbft/tendermint-rs/pull/PRNUM2))
