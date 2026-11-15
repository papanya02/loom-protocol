#!/usr/bin/env bash
set -euo pipefail
cargo fmt
cargo test
cargo build --release
mkdir -p data
[[ -f data/alice.json ]] || cargo run --release -- keygen --key data/alice.json
[[ -f data/bob.json ]] || cargo run --release -- keygen --key data/bob.json
ALICE="$(cargo run --quiet --release -- id --key data/alice.json)"
BOB="$(cargo run --quiet --release -- id --key data/bob.json)"
cat <<EOF
Alice Peer ID: $ALICE
Bob Peer ID:   $BOB

Terminal 2:
cargo run --release -- listen --key data/bob.json --bind 127.0.0.1:7000 --allow-peer-id $ALICE

Terminal 3:
cargo run --release -- send --key data/alice.json --peer 127.0.0.1:7000 --peer-id $BOB --message "Hello from NEXUS 0.2"
EOF
