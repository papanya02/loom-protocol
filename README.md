# loom-protocol — Identity-aware P2P runtime prototype

`loom-protocol` is an experimental modular communication runtime. It is **not** a new cryptographic protocol and does not attempt to replace Noise. It uses the `snow` implementation of the Noise Protocol Framework as its secure-session foundation.

> [!IMPORTANT]
> The design goal is to let applications express *what kind of communication they need*, while **`NEXUS`** owns peer identity, secure sessions, capability negotiation and transport selection.

## What changed from 0.1

- **Stable identity separated from session keys.** An Ed25519 root key determines the `nx2...` Peer ID. A separate X25519 key is used by Noise XX for each secure session.
- **Root-signed Noise key certificate.** After the Noise handshake, peers exchange certificates inside the encrypted channel. Each certificate binds the root identity to the exact Noise static key authenticated by the handshake.
- **Identity pinning now uses the stable root Peer ID**, not the Noise transport key.
- **Capability negotiation is typed** (`id` + `version`) rather than a list of unstructured strings.
- **Transport boundary and path planner introduced.** The current build implements TCP only; the planner API is deliberately small so future QUIC, local and relay adapters can be added without changing application envelopes.
- **Runtime boundaries separated into modules:** identity, crypto, capability, path, transport, wire, node and CLI.
- Handshake timeout and stricter envelope validation added.
- Rust 1.85+ / Edition 2024, matching the current `snow` 0.10 toolchain requirements.

## What is actually implemented

| Capability | Status |
|---|---|
| Noise XX secure handshake | Implemented using `snow` |
| Stable root identity (Ed25519) | Implemented |
| Root-signed Noise key binding | Implemented |
| Stable Peer ID pinning | Implemented |
| Encrypted capability negotiation | Implemented |
| TCP transport adapter | Implemented |
| Intent-aware path planner API | Prototype; only TCP candidate exists |
| Application message + encrypted ACK | Implemented |
| Multiple simultaneous streams | Not implemented |
| QUIC / UDP / relay / NAT traversal | Not implemented |
| DHT / peer discovery | Not implemented |
| Session migration / reconnect continuity | Not implemented |
| Store-and-forward mesh | Not implemented |
| Encrypted key storage / OS keystore | Not implemented |
| Independent security audit | Not performed |

## Requirements

- Rust 1.85 or newer (stable)
- Cargo
- Windows 10/11, Linux or macOS

Install Rust from <https://rustup.rs/>.

Check:

```powershell
rustc --version
cargo --version
```

## Build and test

From the extracted `loom-protocol` directory:

```powershell
cargo fmt
cargo test
cargo build --release
cargo clippy --all-targets --all-features -- -D warnings
```

The first build downloads crates from crates.io.

## Local test on Windows

### Terminal 1 — project directory

```powershell
cargo run --release -- keygen --key data/alice.json
cargo run --release -- keygen --key data/bob.json
cargo run --release -- id --key data/alice.json
cargo run --release -- id --key data/bob.json
```

Copy both Peer IDs.

### Terminal 2 — start Bob

Replace `ALICE_NX2_ID` with Alice's full ID:

```powershell
cargo run --release -- listen --key data/bob.json --bind 127.0.0.1:7000 --allow-peer-id ALICE_NX2_ID
```

### Terminal 3 — Alice sends

Replace `BOB_NX2_ID` with Bob's full ID:

```powershell
cargo run --release -- send --key data/alice.json --peer 127.0.0.1:7000 --peer-id BOB_NX2_ID --message "Hello from NEXUS 0.2" --intent balanced
```

Expected sequence:
1. TCP connection.
2. Noise XX handshake.
3. Encrypted exchange of root-signed identity certificates.
4. Verification that the certificate's Noise key equals the static key authenticated by Noise.
5. Stable root Peer ID pin verification on both sides.
6. Encrypted capability negotiation.
7. Encrypted application message and ACK.

## CLI

```text
nexus keygen --key PATH
nexus id --key PATH
nexus transports
nexus listen --key PATH --bind IP:PORT --allow-peer-id PEER_ID
nexus send --key PATH --peer IP:PORT --peer-id PEER_ID --message TEXT [--intent balanced|low-latency|privacy-preferred|reliable]
```

## Identity model

- Root identity: Ed25519 keypair.
- Stable Peer ID: `nx2` + hex(BLAKE3(`NEXUS-Peer-ID-v2` || root_public_key)).
- Session key: X25519 static key used by Noise XX.
- Binding: root key signs a domain-separated certificate containing the Noise public key.
- Authentication: Noise proves possession of the session key; the encrypted certificate proves that the stable root identity authorized that key; the configured Peer ID pin decides whether the identity is trusted.

The root key and Noise key are currently stored together in plaintext JSON. Protect the file and do not commit it to Git. Key rotation, revocation, recovery and encrypted keystore integration are future work.

## Security and maturity

This is an engineering prototype, not production-ready infrastructure. `snow` supplies Noise primitives; NEXUS adds identity binding and application-level behavior. Neither this project nor its protocol design has undergone an independent security audit.

Do not use it for cryptocurrency custody, production financial operations or sensitive real-world communications. Do not expose the listener to the public Internet.

## Architecture

See:
- `docs/ARCHITECTURE.md`
- `docs/IDENTITY.md`
- `docs/ROADMAP.md`
- `docs/THREAT_MODEL.md`
- `docs/RFC-0002.md`

## References

- Noise Protocol Framework: <https://noiseprotocol.org/noise.html>
- Snow Rust implementation: <https://github.com/mcginty/snow>
- Snow API docs: <https://docs.rs/snow/latest/snow/>
- Ed25519: <https://www.rfc-editor.org/rfc/rfc8032>

## License

Apache-2.0.
