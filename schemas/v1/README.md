# LIMEN local API v1

This directory is the language-neutral source for the local contract between
LIMEN Core and its clients. The transport is defined by D-002: UTF-8 JSON,
prefixed by an unsigned 32-bit little-endian frame length, with a maximum frame
size of 1 MiB.

Every message carries `api_major` and `message_version`. Requests also carry a
portable `request_id`; session events carry a monotonically increasing
`sequence` and their `session_id`.

The first client message on each connection is a handshake containing the
ephemeral secret passed through an operating-system channel. It declares
whether that connection carries commands or the dedicated event subscription.
An incompatible major version, invalid secret, unauthorized capability,
oversized frame or invalid payload fails closed before dispatch.

`limen-api.schema.json` covers the M2 surface. The Rust types in
`crates/contracts` must evolve atomically with this schema.
