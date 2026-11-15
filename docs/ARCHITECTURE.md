# NEXUS 0.2 Architecture

## Design statement

NEXUS is an identity-aware communication runtime, not a replacement for Noise and not yet a complete decentralized network. The prototype separates concerns so the next implementations can be added behind stable module boundaries.

```text
Applications
    |
NEXUS Runtime (node.rs)
    |-- Identity (identity.rs)
    |-- Secure sessions (crypto.rs -> snow)
    |-- Capability negotiation (capability.rs)
    |-- Path policy (path.rs)
    |-- Transport adapters (transport.rs; TCP implemented)
    `-- Wire envelopes (wire.rs)
```

## Module responsibilities

- `identity`: stable Ed25519 root identity, Peer ID, Noise-key certificate.
- `crypto`: Noise XX handshake, encrypted certificate exchange, secure JSON records.
- `capability`: typed capability offers and exact-version intersection.
- `path`: intent and candidate scoring. It is currently a deterministic planning API, not a network measurement engine.
- `transport`: transport boundary and TCP connector.
- `wire`: bounded TCP records and versioned application envelope.
- `node`: executable runtime workflow.
- `cli`: human-operated test interface.

## Trust and identity

The stable identity is an Ed25519 root public key. A separate Noise X25519 key protects sessions. The root signs a domain-separated certificate binding the Noise public key to the root identity. The certificate is sent only after Noise transport encryption is established.

The receiver verifies:
1. Noise XX completed.
2. The certificate's Noise public key equals `get_remote_static()` from the completed handshake.
3. The root signature is valid.
4. The Peer ID is derived from the root public key.
5. The derived Peer ID equals the locally configured expected peer ID.

This is a pin-based trust model. There is no global identity directory or automatic trust network.

## Path planning

Applications choose a connection intent (`balanced`, `low-latency`, `privacy-preferred`, `reliable`). A planner scores available candidates. The current build has only a TCP candidate, so all intents currently resolve to TCP. This API is a seam for future transports, not a claim that alternate paths work today.

## Wire format

The prototype uses a 2-byte big-endian TCP record length followed by one Noise message. Application messages are encrypted JSON envelopes with:
- protocol version
- message ID
- message kind
- logical stream ID
- payload

JSON is intentionally used for development clarity. A stable interoperable release should define canonical CBOR or another rigorously specified binary encoding, canonicalization rules, limits and test vectors.

## Concurrency

The listener currently accepts one peer and completes one request/response. Multiplexing and concurrent session management are not implemented.
