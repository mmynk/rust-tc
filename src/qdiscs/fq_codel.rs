use serde::{Deserialize, Serialize};

use crate::{errors::Error, TcOption};

/// Defined in `include/uapi/linux/sch_fq_codel.c`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FqCodel {
    pub target: u32,
    pub limit: u32,
    pub interval: u32,
    pub ecn: u32,
    pub flows: u32,
    pub quantum: u32,
    pub ce_threshold: u32,
    pub drop_batch_size: u32,
    pub memory_limit: u32,
}

impl FqCodel {
    pub fn new(opts: Vec<TcOption>) -> Self {
        unmarshal_fq_codel(opts)
    }
}

/// Defined in `include/uapi/linux/pkt_sched.h` as `struct tc_fq_codel_xstats`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FqCodelXStats {
    pub maxpacket: u32,
    pub drop_overlimit: u32,
    pub ecn_mark: u32,
    pub new_flow_count: u32,
    pub new_flows_len: u32,
    pub old_flows_len: u32,
    pub ce_mark: u32,
    pub memory_usage: u32,
    pub drop_overmemory: u32,
}

impl FqCodelXStats {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        unmarshal_fq_codel_xstats(bytes)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum TcaFqCodel {
    #[default]
    Unspec = 0,
    Target,
    Limit,
    Interval,
    Ecn,
    Flows,
    Quantum,
    CeThreshold,
    DropBatchSize,
    MemoryLimit,
    Max,
}

impl From<u16> for TcaFqCodel {
    fn from(v: u16) -> Self {
        match v {
            0 => TcaFqCodel::Unspec,
            1 => TcaFqCodel::Target,
            2 => TcaFqCodel::Limit,
            3 => TcaFqCodel::Interval,
            4 => TcaFqCodel::Ecn,
            5 => TcaFqCodel::Flows,
            6 => TcaFqCodel::Quantum,
            7 => TcaFqCodel::CeThreshold,
            8 => TcaFqCodel::DropBatchSize,
            9 => TcaFqCodel::MemoryLimit,
            _ => TcaFqCodel::Max,
        }
    }
}

fn unmarshal_fq_codel(opts: Vec<TcOption>) -> FqCodel {
    let mut fq = FqCodel::default();

    for opt in opts {
        let kind = TcaFqCodel::from(opt.kind);
        if opt.bytes.len() < 4 {
            // TODO: log error
            continue;
        }
        let value = u32::from_ne_bytes(opt.bytes[..4].try_into().unwrap());
        match kind {
            TcaFqCodel::Target => fq.target = value,
            TcaFqCodel::Limit => fq.limit = value,
            TcaFqCodel::Interval => fq.interval = value,
            TcaFqCodel::Ecn => fq.ecn = value,
            TcaFqCodel::Flows => fq.flows = value,
            TcaFqCodel::Quantum => fq.quantum = value,
            TcaFqCodel::CeThreshold => fq.ce_threshold = value,
            TcaFqCodel::DropBatchSize => fq.drop_batch_size = value,
            TcaFqCodel::MemoryLimit => fq.memory_limit = value,
            _ => (),
        }
    }

    fq
}

fn unmarshal_fq_codel_xstats(bytes: &[u8]) -> Result<FqCodelXStats, Error> {
    if bytes.len() < 40 {
        return Err(Error::Parse(
            "FqCodel XStats requires 40 bytes".to_string(),
        ));
    }
    let buf: [u8; 4] = bytes[..4]
        .try_into()
        .map_err(|_| Error::Parse("Failed to extract FqCodel XStats kind".to_string()))?;
    let kind = u32::from_ne_bytes(buf);
    if kind == 0 {
        bincode::deserialize(&bytes[4..]).map_err(|e| TcError::Parse(e.to_string()))
    } else {
        Err(Error::Parse(format!(
            "FqCodel XStats has unidentified kind: {kind}"
        )))
    }
}
