use crate::qdiscs::{
    clsact::Clsact,
    fq_codel::{FqCodel, FqCodelXStats},
};

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
}

#[derive(Debug, PartialEq)]
pub enum XStats {
    FqCodel(FqCodelXStats),
}
