$ErrorActionPreference = "Stop"

cargo fmt
cargo test
cargo build --release
New-Item -ItemType Directory -Force -Path "data" | Out-Null

if (-not (Test-Path "data/alice.json")) { cargo run --release -- keygen --key data/alice.json }
if (-not (Test-Path "data/bob.json")) { cargo run --release -- keygen --key data/bob.json }

$alice = (cargo run --quiet --release -- id --key data/alice.json).Trim()
$bob = (cargo run --quiet --release -- id --key data/bob.json).Trim()

Write-Host ""
Write-Host "Alice Peer ID: $alice"
Write-Host "Bob Peer ID:   $bob"
Write-Host ""
Write-Host "Terminal 2 (Bob listener):"
Write-Host "cargo run --release -- listen --key data/bob.json --bind 127.0.0.1:7000 --allow-peer-id $alice"
Write-Host ""
Write-Host "Terminal 3 (Alice sender):"
Write-Host "cargo run --release -- send --key data/alice.json --peer 127.0.0.1:7000 --peer-id $bob --message `"Hello from NEXUS 0.2`""
