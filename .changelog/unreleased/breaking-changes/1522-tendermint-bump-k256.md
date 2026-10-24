- `[tendermint]`: Update `k256` to v0.14 and `signature` to v3
  to pull in a patched version of `crypto-bigint` (RUSTSEC-2026-0387).
  The re-exported secp256k1 types (`public_key::Secp256k1`, `signature::Secp256k1Signature`,
  `private_key::Secp256k1` and `crypto::default::ecdsa_secp256k1`) now come from `k256` v0.14.
  ([\#1522](https://github.com/cometbft/tendermint-rs/pull/1522))
