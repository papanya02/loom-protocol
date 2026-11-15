# Identity model

## Stable root identity

Each node has an Ed25519 root keypair. The public key is the stable cryptographic identity anchor. Peer ID:

`nx2 || hex(BLAKE3("NEXUS-Peer-ID-v2" || root_public_key))`

The Peer ID is not a username, legal identity or proof of reputation.

## Session key certificate

Each node also has a Noise X25519 static keypair. The root signs:

`"NEXUS-NOISE-KEY-CERT-v1" || root_public_key || noise_static_public_key`

After Noise XX finishes, peers exchange this certificate inside the encrypted transport. The remote certificate is accepted only if:
- its Ed25519 signature verifies;
- its Noise key exactly equals the remote static key revealed by the Noise handshake;
- its derived Peer ID equals the certificate Peer ID;
- its Peer ID matches the local pin/allow-list.

This permits future Noise key rotation without changing the root Peer ID, provided a new key is certified by the same root. Rotation/revocation commands and certificate expiry are not implemented yet.

## Operational warning

Both private keys currently live in a plaintext JSON file. Use a private local directory, backups with strong protection, and never upload the file to source control or send it to another person.
