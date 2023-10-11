use netlink_packet_utils::nla::{self, Nla};

use crate::constants::ATTR_LEN;

#[derive(Clone, Debug, Default)]
pub struct FqCodel {
    target: u32,
    limit: u32,
    interval: u32,
    ecn: u32,
    flows: u32,
    quantum: u32,
    ce_threshold: u32,
    drop_batch_size: u32,
    memory_limit: u32,
}

impl FqCodel {
    pub fn new(nla: Vec<&nla::DefaultNla>) -> Self {
        unmarshal_fq_codel(nla)
    }
}

#[derive(Clone, Debug, Default)]
pub struct FqCodelXStats {
    maxpacket: u32,
    drop_overlimit: u32,
    ecn_mark: u32,
    new_flow_count: u32,
    new_flows_len: u32,
    old_flows_len: u32,
    ce_mark: u32,
    memory_usage: u32,
    drop_overmemory: u32,
}

impl FqCodelXStats {
    pub fn new(bytes: [u8; 40]) -> Self {
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

fn unmarshal_fq_codel_xstats(bytes: [u8; 40]) -> FqCodelXStats {
    let mut fq = FqCodelXStats::default();

    let buf: [u8; 4] = bytes[..4].try_into().unwrap();
    let kind = u32::from_ne_bytes(buf);
    if kind == 0 {
        fq.maxpacket = u32::from_ne_bytes(bytes[4..8].try_into().unwrap());
        fq.drop_overlimit = u32::from_ne_bytes(bytes[8..12].try_into().unwrap());
        fq.ecn_mark = u32::from_ne_bytes(bytes[12..16].try_into().unwrap());
        fq.new_flow_count = u32::from_ne_bytes(bytes[16..20].try_into().unwrap());
        fq.new_flows_len = u32::from_ne_bytes(bytes[20..24].try_into().unwrap());
        fq.old_flows_len = u32::from_ne_bytes(bytes[24..28].try_into().unwrap());
        fq.ce_mark = u32::from_ne_bytes(bytes[28..32].try_into().unwrap());
        fq.memory_usage = u32::from_ne_bytes(bytes[32..36].try_into().unwrap());
        fq.drop_overmemory = u32::from_ne_bytes(bytes[36..40].try_into().unwrap());
    }

    fq
}
