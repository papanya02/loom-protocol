# NEXUS roadmap

## 0.2 — identity-aware secure runtime (this prototype)
- Stable root identity separated from Noise session key.
- Root-signed session-key certificate.
- Peer ID pinning against stable root identity.
- Typed capability negotiation.
- TCP adapter and transport-selection seam.
- Single request/response workflow.

## 0.3 — runtime and protocol hardening
- Extract modules into a Cargo workspace only after APIs stabilize.
- Define canonical binary wire encoding and version negotiation.
- Add request correlation, stream lifecycle, flow control and multiplexing.
- Add handshake and application-level fuzz targets.
- Add property tests, test vectors and interoperability tests.
- Add structured errors, tracing, rate limits and bounded concurrency.
- Add encrypted key storage and key rotation/revocation.

## 0.4 — multiple transports
- QUIC adapter with certificate/identity binding.
- Local-network discovery adapter.
- Transport health measurements and policy-driven path selection.
- Explicit downgrade prevention and transport-binding rules.

## 0.5 — decentralized discovery and relay
- Signed peer records with expiry.
- DHT/discovery design with poisoning resistance.
- Relay protocol with end-to-end encryption preserved.
- NAT traversal experiments.
- Abuse controls and resource quotas.

## 0.6 — continuity and offline operation
- Session resumption design with forward-secrecy analysis.
- Connection migration across network changes.
- Store-and-forward queues with expiry and delivery semantics.
- Mesh forwarding with hop limits and loop prevention.

## 1.0 gate
No production claim until protocol specification, independent security review, fuzzing, interoperability suite, operational key management, denial-of-service controls and documented upgrade/rollback behavior are complete.
