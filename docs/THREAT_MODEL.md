# Threat Model — NEXUS 0.2

## Protected assets
- Confidentiality and integrity of messages in a completed Noise transport session.
- Stable root identity and its authorization of a Noise session key.
- Local root and Noise private keys.

## Addressed threats
- Passive observation and active modification of session traffic: mitigated by Noise XX transport encryption.
- Impersonation using an unapproved Noise static key: mitigated by root-signed key certificate plus Peer ID pinning.
- Substitution of the certificate's Noise key: rejected because it must equal the Noise handshake remote static key.
- Corrupt or oversized TCP records: bounded record framing.
- Stalled handshake: a 15-second timeout at the node workflow.

## Not addressed
- Compromised endpoint or stolen plaintext key file.
- Traffic analysis, metadata privacy or anonymity.
- DDoS beyond the single-session prototype and basic size/time limits.
- Key revocation, expiry, recovery and secure rotation workflow.
- DHT poisoning, malicious relays or Sybil resistance.
- Replay semantics across sessions, application idempotency and full stream sequencing.
- Formal verification or independent audit.

## Security posture
This is an experimental engineering prototype. Correct use of Noise and Ed25519 does not make the surrounding protocol automatically secure. Do not use it for custody, financial transactions or production-sensitive communication.
