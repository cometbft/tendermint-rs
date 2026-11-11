- `[tendermint-proto]` Replace the unmaintained `subtle-encoding` dependency with a
  vendored `tendermint_proto::encoding` module (constant-time hex and Base64, plus a
  Bech32 encoder). Like `serializers`, the module is an internal implementation detail.
- `[tendermint]` Drop the `subtle-encoding` dependency. The `Error::subtle_encoding`
  constructor and `ErrorDetail::SubtleEncoding` variant are renamed to `Error::encoding` /
  `ErrorDetail::Encoding`, and wrap `tendermint_proto::encoding::Error`.
- `[tendermint-rpc]` Drop the `subtle-encoding` dependency in favour of
  `tendermint_proto::encoding`.
