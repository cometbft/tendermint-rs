- `[tendermint-rpc]`: Update `reqwest` to v0.12 and `async-tungstenite` to v0.28
  to pull in patched versions of `h2`, `rustls` and `rustls-webpki`
  (RUSTSEC-2026-0258, RUSTSEC-2026-0346, RUSTSEC-2026-0349, RUSTSEC-2026-0049,
  RUSTSEC-2026-0098, RUSTSEC-2026-0099, RUSTSEC-2026-0104, RUSTSEC-2026-0382).
  `HttpClient::builder().client(...)` and `HttpClient::new_from_parts` now take a
  `reqwest` v0.12 `Client`, and the re-exported `WebSocketConfig` comes from `tungstenite` v0.24.
  The `websocket-client` feature now enables the `ring` crypto provider for `rustls`.
  ([\#1522](https://github.com/cometbft/tendermint-rs/pull/1522))
- `[tendermint-light-client-js]`: Relax the `js-sys` pin from `=0.3.70` to `0.3.70`
  so that `reqwest` can resolve to a version without the vulnerable `quinn-proto`.
  ([\#1522](https://github.com/cometbft/tendermint-rs/pull/1522))
