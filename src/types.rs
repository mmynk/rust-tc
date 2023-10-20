use serde::{Deserialize, Serialize};

use crate::{errors::TcError, Clsact, FqCodel, FqCodelXStats, Htb, HtbGlob, HtbXstats};

#[derive(Debug, Default)]
pub struct Tc {
    pub msg: TcMessage,
    pub attr: Attribute,
}

#[derive(Debug, Default)]
pub struct TcMessage {
    pub index: u32,
    pub handle: u32,
    pub parent: u32,
}

#[derive(Debug, Default)]
pub struct Attribute {
    pub kind: Option<String>,
    pub stats: Option<Stats>,
    pub stats2: Option<Stats2>,
    pub qdisc: Option<QDisc>,
    pub class: Option<Class>,
    pub xstats: Option<XStats>,
}

#[derive(Debug, Default)]
pub struct Stats {
    pub bytes: u64,
    pub packets: u32,
    pub drops: u32,
    pub overlimits: u32,
    pub bps: u32,
    pub pps: u32,
    pub qlen: u32,
    pub backlog: u32,
}

#[derive(Debug, Default)]
pub struct StatsBasic {
    pub bytes: u64,
    pub packets: u32,
}

#[derive(Debug, Default)]
pub struct StatsQueue {
    pub qlen: u32,
    pub backlog: u32,
    pub drops: u32,
    pub requeues: u32,
    pub overlimits: u32,
}

#[derive(Debug, Default)]
pub struct Stats2 {
    pub basic: Option<StatsBasic>,
    pub queue: Option<StatsQueue>,
    // pub app: Option<StatsApp>,
}

#[derive(Debug, PartialEq)]
pub enum QDisc {
    FqCodel(FqCodel),
    Clsact(Clsact),
    Htb(HtbGlob),
}

#[derive(Debug, PartialEq)]
pub enum Class {
    Htb(Htb),
}

#[derive(Debug, PartialEq)]
pub enum XStats {
    FqCodel(FqCodelXStats),
    Htb(HtbXstats),
}

/// Defined in `include/uapi/linux/pkt_sched.h` as `struct tc_ratespec`
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct RateSpec {
    pub cell_log: u8,
    pub linklayer: u8,
    pub overhead: u16,
    pub cell_align: u16,
    pub mpu: u16,
    pub rate: u32,
}

pub fn unmarshal_rate_spec(buf: &[u8]) -> Result<RateSpec, TcError> {
    bincode::deserialize(buf).map_err(|e| TcError::UnmarshalStruct(e))
}

/// A subset of structs defined in `include/uapi/linux/if_link.h`.
#[derive(Debug)]
pub struct Link {
    pub index: u32,
    pub name: String,
}
