use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use snow::{params::NoiseParams, Builder};
use std::{fs, io::Write, path::{Path, PathBuf}};

const NOISE_PATTERN: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
const PEER_DOMAIN: &[u8] = b"NEXUS-Peer-ID-v2";
const CERT_DOMAIN: &[u8] = b"NEXUS-NOISE-KEY-CERT-v1";

#[derive(Serialize, Deserialize)]
struct KeyFile {
    format: u32,
    root_private_key_hex: String,
    root_public_key_hex: String,
    noise_private_key_hex: String,
    noise_public_key_hex: String,
}

pub struct Identity {
    pub root_signing_key: SigningKey,
    pub root_public_key: [u8; 32],
    pub noise_private_key: Vec<u8>,
    pub noise_public_key: Vec<u8>,
    pub peer_id: String,
    pub key_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyCertificate {
    pub peer_id: String,
    pub root_public_key_hex: String,
    pub noise_static_public_key_hex: String,
    pub signature_hex: String,
}

pub fn peer_id(root_public_key: &[u8; 32]) -> String {
    let mut input = Vec::with_capacity(PEER_DOMAIN.len() + 32);
    input.extend_from_slice(PEER_DOMAIN);
    input.extend_from_slice(root_public_key);
    format!("nx2{}", hex::encode(blake3::hash(&input).as_bytes()))
}

fn certificate_message(root_public: &[u8; 32], noise_public: &[u8]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(CERT_DOMAIN.len() + 32 + noise_public.len());
    msg.extend_from_slice(CERT_DOMAIN);
    msg.extend_from_slice(root_public);
    msg.extend_from_slice(noise_public);
    msg
}

impl Identity {
    pub fn certificate(&self) -> KeyCertificate {
        let message = certificate_message(&self.root_public_key, &self.noise_public_key);
        let signature = self.root_signing_key.sign(&message);
        KeyCertificate {
            peer_id: self.peer_id.clone(),
            root_public_key_hex: hex::encode(self.root_public_key),
            noise_static_public_key_hex: hex::encode(&self.noise_public_key),
            signature_hex: hex::encode(signature.to_bytes()),
        }
    }
}

pub fn verify_certificate(cert: &KeyCertificate, expected_noise_static: &[u8]) -> Result<String> {
    let root_bytes = hex::decode(&cert.root_public_key_hex).context("invalid root public key hex")?;
    let root: [u8; 32] = root_bytes.try_into().map_err(|_| anyhow::anyhow!("root public key must be 32 bytes"))?;
    let noise = hex::decode(&cert.noise_static_public_key_hex).context("invalid Noise public key hex")?;
    if noise != expected_noise_static {
        bail!("identity certificate Noise key does not match the key authenticated by Noise XX");
    }

    let key = VerifyingKey::from_bytes(&root).context("invalid Ed25519 root public key")?;
    let sig_bytes = hex::decode(&cert.signature_hex).context("invalid certificate signature hex")?;
    let signature = Signature::from_slice(&sig_bytes).context("invalid certificate signature")?;
    let message = certificate_message(&root, &noise);
    key.verify(&message, &signature).context("Noise key certificate signature verification failed")?;

    let derived_id = peer_id(&root);
    if derived_id != cert.peer_id {
        bail!("certificate Peer ID does not match root public key");
    }
    Ok(derived_id)
}

pub fn generate(path: &Path) -> Result<Identity> {
    if path.exists() {
        bail!("key file already exists: {} (refusing to overwrite)", path.display());
    }
    let root = SigningKey::generate(&mut OsRng);
    let root_public_key = root.verifying_key().to_bytes();

    let params: NoiseParams = NOISE_PATTERN.parse().context("invalid Noise pattern")?;
    let noise_pair = Builder::new(params).generate_keypair().context("Noise key generation failed")?;

    let key_file = KeyFile {
        format: 2,
        root_private_key_hex: hex::encode(root.to_bytes()),
        root_public_key_hex: hex::encode(root_public_key),
        noise_private_key_hex: hex::encode(&noise_pair.private),
        noise_public_key_hex: hex::encode(&noise_pair.public),
    };

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| format!("cannot create {}", parent.display()))?;
        }
    }
    let bytes = serde_json::to_vec_pretty(&key_file)?;
    write_private_file(path, &bytes)?;

    Ok(Identity {
        root_signing_key: root,
        root_public_key,
        noise_private_key: noise_pair.private,
        noise_public_key: noise_pair.public,
        peer_id: peer_id(&root_public_key),
        key_path: path.to_path_buf(),
    })
}

pub fn load(path: &Path) -> Result<Identity> {
    let bytes = fs::read(path).with_context(|| format!("cannot read key file {}", path.display()))?;
    let file: KeyFile = serde_json::from_slice(&bytes).context("invalid key file JSON")?;
    if file.format != 2 {
        bail!("unsupported key file format {}; NEXUS 0.2 requires format 2", file.format);
    }

    let root_secret: [u8; 32] = hex::decode(&file.root_private_key_hex)?
        .try_into().map_err(|_| anyhow::anyhow!("root private key must be 32 bytes"))?;
    let root_public: [u8; 32] = hex::decode(&file.root_public_key_hex)?
        .try_into().map_err(|_| anyhow::anyhow!("root public key must be 32 bytes"))?;
    let root_signing_key = SigningKey::from_bytes(&root_secret);
    if root_signing_key.verifying_key().to_bytes() != root_public {
        bail!("root public key does not match root private key");
    }

    let noise_private_key = hex::decode(&file.noise_private_key_hex)?;
    let noise_public_key = hex::decode(&file.noise_public_key_hex)?;
    if noise_private_key.len() != 32 || noise_public_key.len() != 32 {
        bail!("Noise X25519 keys must be 32 bytes");
    }

    Ok(Identity {
        root_signing_key,
        root_public_key: root_public,
        noise_private_key,
        noise_public_key,
        peer_id: peer_id(&root_public),
        key_path: path.to_path_buf(),
    })
}

fn write_private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600)
            .open(path).with_context(|| format!("cannot create key file {}", path.display()))?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
            .with_context(|| format!("cannot create key file {}", path.display()))?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_peer_id_and_certificate_verification() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("node.json");
        let identity = generate(&path).unwrap();
        let cert = identity.certificate();
        let verified = verify_certificate(&cert, &identity.noise_public_key).unwrap();
        assert_eq!(verified, identity.peer_id);
        assert!(verify_certificate(&cert, &[9u8; 32]).is_err());
    }

    #[test]
    fn key_file_round_trip_and_root_key_consistency() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("node.json");
        let first = generate(&path).unwrap();
        let second = load(&path).unwrap();
        assert_eq!(first.peer_id, second.peer_id);
        assert_eq!(first.root_public_key, second.root_public_key);
    }
}
