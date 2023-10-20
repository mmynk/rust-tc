use netlink_packet_utils::nla::{self, Nla};
use serde::{Deserialize, Serialize};

use crate::{constants::ATTR_LEN, errors::TcError};

/// FQ_Codel (Fair Queuing Controlled Delay) is queuing discipline
/// that combines Fair Queuing with the CoDel AQM scheme.
/// FQ_Codel uses a stochastic model to classify incoming packets into
/// different flows and is used to provide a fair share of the
/// bandwidth to all the flows using the queue.
/// Each such flow is managed by the CoDel queuing discipline.
/// Reordering within a flow is avoided since Codel internally uses a FIFO queue.
///
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
    pub fn new(nla: Vec<&nla::DefaultNla>) -> Self {
        unmarshal_fq_codel(nla)
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
    pub fn new(bytes: &[u8]) -> Result<Self, TcError> {
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

fn unmarshal_fq_codel(nlas: Vec<&nla::DefaultNla>) -> FqCodel {
    let mut fq = FqCodel::default();

    let mut buf = [0u8; ATTR_LEN];
    for nla in nlas {
        let kind = TcaFqCodel::from(nla.kind());
        nla.emit_value(&mut buf[..]);
        match kind {
            TcaFqCodel::Target => fq.target = u32::from_ne_bytes(buf),
            TcaFqCodel::Limit => fq.limit = u32::from_ne_bytes(buf),
            TcaFqCodel::Interval => fq.interval = u32::from_ne_bytes(buf),
            TcaFqCodel::Ecn => fq.ecn = u32::from_ne_bytes(buf),
            TcaFqCodel::Flows => fq.flows = u32::from_ne_bytes(buf),
            TcaFqCodel::Quantum => fq.quantum = u32::from_ne_bytes(buf),
            TcaFqCodel::CeThreshold => fq.ce_threshold = u32::from_ne_bytes(buf),
            TcaFqCodel::DropBatchSize => fq.drop_batch_size = u32::from_ne_bytes(buf),
            TcaFqCodel::MemoryLimit => fq.memory_limit = u32::from_ne_bytes(buf),
            _ => (),
        }
    }

    fq
}

fn unmarshal_fq_codel_xstats(bytes: &[u8]) -> Result<FqCodelXStats, TcError> {
    if bytes.len() < 40 {
        return Err(TcError::InvalidAttribute(
            "FqCodel XStats requires 40 bytes".to_string(),
        ));
    }
    let buf: [u8; 4] = bytes[..4]
        .try_into()
        .map_err(|_| TcError::Decode("Failed to extract FqCodel XStats kind".to_string()))?;
    let kind = u32::from_ne_bytes(buf);
    if kind == 0 {
        bincode::deserialize(bytes).map_err(|e| TcError::UnmarshalStruct(e))
    } else {
        Err(TcError::InvalidAttribute(format!(
            "FqCodel XStats has unidentified kind: {kind}"
        )))
    }
}
