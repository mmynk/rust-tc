use serde::{Deserialize, Serialize};

use crate::{errors::Error};
use crate::class::{Htb, HtbGlob, HtbXstats};
use crate::qdiscs::{Clsact, FqCodel, FqCodelXStats};

/// This struct is an intermediate representation for netlink `tc` messages.
/// Any downstream structs should be constructed into this struct.
#[derive(Debug, Default, PartialEq)]
pub struct TcMsg {
    pub header: TcHeader,
    pub attrs: Vec<TcAttr>,
}

#[derive(Debug, Default, PartialEq)]
pub struct TcHeader {
    pub index: i32,
    pub handle: u32,
    pub parent: u32,
}

#[derive(Debug, PartialEq)]
pub enum TcAttr {
    Unspec(Vec<u8>),
    Kind(String),
    Options(Vec<TcOption>),
    Stats(Vec<u8>),
    Xstats(Vec<u8>),
    Rate(Vec<u8>),
    Fcnt(Vec<u8>),
    Stats2(Vec<TcStats2>),
    Stab(Vec<u8>),
    Pad(Vec<u8>),
    Chain(Vec<u8>),
    HwOffload(u8),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TcOption {
    pub kind: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum TcStats2 {
    StatsBasic(Vec<u8>),
    StatsQueue(Vec<u8>),
    StatsApp(Vec<u8>),
}

/// This struct is an intermediate representation for netlink `link` messages.
/// Any downstream structs should be constructed into this struct.
#[derive(Debug)]
pub struct LinkMsg {
    pub header: LinkHeader,
    pub attr: LinkAttr,
}

#[derive(Debug)]
pub struct LinkHeader {
    pub index: u32,
}

#[derive(Debug)]
pub struct LinkAttr {
    pub name: String,
}

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
    pub kind: String,
    pub stats: Option<Stats>,
    pub stats2: Option<Stats2>,
    pub qdisc: Option<QDisc>,
    pub class: Option<Class>,
    pub xstats: Option<XStats>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StatsBasic {
    pub bytes: u64,
    pub packets: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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

pub fn unmarshal_rate_spec(buf: &[u8]) -> Result<RateSpec, Error> {
    bincode::deserialize(buf).map_err(Error::UnmarshalStruct)
}

/// A subset of structs defined in `include/uapi/linux/if_link.h`.
#[derive(Debug)]
pub struct Link {
    pub index: u32,
    pub name: String,
}
