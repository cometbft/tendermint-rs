*October 23rd, 2026*

This release updates the TLS stack used by the `tendermint-rpc` HTTP and
WebSocket clients to `rustls` 0.23, with `rustls-webpki` 0.103.16 or later,
so that certificate chains through distrusted CA keys are rejected. All users
of the RPC clients are advised to update.

It also makes `Verifier::verify` take `&self`, allowing stateful signature
verifiers. See the breaking changes below for details.
