- `[tendermint]`: Update `k256` to v0.14 and `signature` to v3
  to pull in a patched version of `crypto-bigint` (RUSTSEC-2026-0387).
  The re-exported secp256k1 types (`public_key::Secp256k1`, `signature::Secp256k1Signature`,
  `private_key::Secp256k1` and `crypto::default::ecdsa_secp256k1`) now come from `k256` v0.14.
  The `signature` dependency is now only pulled in by the `secp256k1` feature.
  The minimum supported Rust version is now 1.82 (required by `prost` v0.14),
  or 1.85 when the `secp256k1` feature is enabled (required by `k256` v0.14 and `signature` v3).
  ([\#1522](https://github.com/cometbft/tendermint-rs/pull/1522))
