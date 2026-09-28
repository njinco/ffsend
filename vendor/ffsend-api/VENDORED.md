# Local ffsend-api copy

This directory started from the published `ffsend-api` 0.7.3 crate (MIT
license; see `LICENSE`). It is kept in this fork because Send v3 uploads use
the old `websocket` crate, which brings in `hyper` 0.10.

Local changes:

- Replace the Send v3 WebSocket client with `tungstenite` 0.26.2.
- Preserve the `ffsend` subprotocol and optional HTTP Basic authentication on
  the WebSocket upgrade.
- Keep the Send v3 metadata, encrypted header, chunk, footer, and response
  sequence in `src/action/upload.rs`.

The root `tests/send3_upload.rs` checks this upload protocol with a local mock
server. When updating this copy, compare against a new `ffsend-api` release and
run the local tests and a disposable round trip against the personal Send host.
