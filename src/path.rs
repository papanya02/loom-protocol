use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionIntent {
    Balanced,
    LowLatency,
    PrivacyPreferred,
    Reliable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Tcp,
    Quic,
    Local,
    Relay,
}

#[derive(Debug, Clone)]
pub struct PathCandidate {
    pub transport: TransportKind,
    pub available: bool,
    pub estimated_latency_ms: Option<u32>,
    pub privacy_score: u8,
    pub reliability_score: u8,
}

pub fn choose_path(intent: ConnectionIntent, candidates: &[PathCandidate]) -> Result<TransportKind> {
    let mut usable = candidates.iter().filter(|c| c.available).collect::<Vec<_>>();
    if usable.is_empty() { bail!("no available transport path"); }

    usable.sort_by_key(|c| {
        let latency = c.estimated_latency_ms.unwrap_or(10_000).min(10_000);
        let score = match intent {
            ConnectionIntent::LowLatency => (latency as i64) * 4 - (c.reliability_score as i64),
            ConnectionIntent::PrivacyPreferred => -(c.privacy_score as i64) * 100 + latency as i64,
            ConnectionIntent::Reliable => -(c.reliability_score as i64) * 100 + latency as i64,
            ConnectionIntent::Balanced => latency as i64 - (c.privacy_score as i64) * 10 - (c.reliability_score as i64) * 10,
        };
        score
    });
    Ok(usable[0].transport)
}

pub fn current_candidates() -> Vec<PathCandidate> {
    // TCP is the only implemented adapter in this MVP. Other entries are
    // deliberately absent rather than advertised as working transports.
    vec![PathCandidate {
        transport: TransportKind::Tcp,
        available: true,
        estimated_latency_ms: None,
        privacy_score: 0,
        reliability_score: 50,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_selects_available_path() {
        let candidates = vec![
            PathCandidate { transport: TransportKind::Tcp, available: true, estimated_latency_ms: Some(40), privacy_score: 1, reliability_score: 50 },
            PathCandidate { transport: TransportKind::Quic, available: true, estimated_latency_ms: Some(10), privacy_score: 1, reliability_score: 50 },
        ];
        assert_eq!(choose_path(ConnectionIntent::LowLatency, &candidates).unwrap(), TransportKind::Quic);
    }
}
