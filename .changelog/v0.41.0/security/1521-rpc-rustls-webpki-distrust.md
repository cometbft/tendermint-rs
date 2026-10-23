- `[tendermint-rpc]` Update the HTTP and WebSocket clients to `rustls` 0.23 and
  require `rustls-webpki` 0.103.16 or later, which rejects certificate chains
  through the eMudhra intermediate CA keys distrusted by `rustls-webpki`
  ([\#1521](https://github.com/cometbft/tendermint-rs/pull/1521))
