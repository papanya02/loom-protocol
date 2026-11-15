# from NEXUS 0.1 to 0.2

NEXUS 0.2 is intentionally not wire-compatible with 0.1.

- Key file format changed from 1 to 2.
- Peer ID changed from `nx1...` (Noise key-derived) to `nx2...` (Ed25519 root-derived).
- The old key files cannot be used by this version. Generate new identities with `nexus keygen`.
- The handshake prologue and application protocol version changed.
- Do not delete old identity backups until you have confirmed you no longer need the 0.1 test identities.
