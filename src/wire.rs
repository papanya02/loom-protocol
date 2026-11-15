use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MAX_RECORD: usize = 65_535;
pub const MAX_PLAINTEXT: usize = 60_000;
pub const PROTOCOL_VERSION: &str = "nexus/0.2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub protocol: String,
    pub message_id: u64,
    pub kind: String,
    pub stream_id: u32,
    pub payload: serde_json::Value,
}

pub async fn write_record<W: AsyncWrite + Unpin>(writer: &mut W, bytes: &[u8]) -> Result<()> {
    if bytes.is_empty() || bytes.len() > MAX_RECORD {
        bail!("record size out of bounds");
    }
    writer.write_u16(bytes.len() as u16).await?;
    writer.write_all(bytes).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_record<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
    let len = reader.read_u16().await.context("failed to read record length")? as usize;
    if len == 0 || len > MAX_RECORD {
        bail!("invalid record length: {len}");
    }
    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).await.context("truncated record")?;
    Ok(data)
}
