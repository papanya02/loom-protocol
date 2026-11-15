use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{crypto::{receive_secure_json, send_secure_json}, wire::{Envelope, PROTOCOL_VERSION}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub id: String,
    pub version: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityOffer {
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAgreement {
    pub capabilities: Vec<Capability>,
}

pub fn local_capabilities() -> Vec<Capability> {
    vec![
        Capability { id: "nexus/base".into(), version: 2 },
        Capability { id: "nexus/message".into(), version: 1 },
        Capability { id: "nexus/ping".into(), version: 1 },
    ]
}

pub async fn negotiate<S>(
    stream: &mut S,
    transport: &mut snow::TransportState,
) -> Result<Vec<Capability>>
where S: AsyncRead + AsyncWrite + Unpin {
    let local = CapabilityOffer { capabilities: local_capabilities() };
    send_secure_json(stream, transport, &Envelope {
        protocol: PROTOCOL_VERSION.into(),
        message_id: 1,
        kind: "capability_offer".into(),
        stream_id: 0,
        payload: serde_json::to_value(&local)?,
    }).await?;

    let remote: Envelope = receive_secure_json(stream, transport).await?;
    if remote.protocol != PROTOCOL_VERSION || remote.kind != "capability_offer" || remote.stream_id != 0 {
        bail!("invalid capability offer envelope");
    }
    let remote: CapabilityOffer = serde_json::from_value(remote.payload).context("malformed capability offer")?;
    let mut shared = local.capabilities.into_iter()
        .filter(|ours| remote.capabilities.iter().any(|theirs| ours == theirs))
        .collect::<Vec<_>>();
    shared.sort_by(|a, b| a.id.cmp(&b.id).then(a.version.cmp(&b.version)));
    if !shared.iter().any(|c| c.id == "nexus/base" && c.version == 2) {
        bail!("no compatible NEXUS base capability");
    }
    Ok(shared)
}
