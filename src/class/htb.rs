use serde::{Deserialize, Serialize};

use crate::{errors::Error, types::*};

/// Defined in `include/uapi/linux/pkt_sched.h`.
#[derive(Default, Debug, PartialEq)]
pub struct Htb {
    pub parms: Option<HtbOpt>,
    pub init: Option<HtbGlob>,
    pub ctab: Vec<u8>,
    pub rtab: Vec<u8>,
    pub direct_qlen: Option<u32>,
    pub rate64: Option<u64>,
    pub ceil64: Option<u64>,
}

/// Defined in `include/uapi/linux/pkt_sched.h` as `struct tc_htb_opt`.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct HtbOpt {
    pub rate: RateSpec,
    pub ceil: RateSpec,
    pub buffer: u32,
    pub cbuffer: u32,
    pub quantum: u32,
    pub level: u32,
    pub prio: u32,
}

/// Defined in `include/uapi/linux/pkt_sched.h` as `struct tc_htb_glob`.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct HtbGlob {
    pub version: u32,
    pub rate2quantum: u32,
    pub defcls: u32,
    pub debug: u32,
    pub direct_pkts: u32,
}

/// Defined in `include/uapi/linux/pkt_sched.h` as `struct tc_htb_xstats`.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct HtbXstats {
    pub lends: u32,
    pub borrows: u32,
    pub giants: u32,
    pub tokens: u32,
    pub ctokens: u32,
}

pub enum TcaHtb {
    Unspec = 0,
    Parms,
    Init,
    Ctab,
    Rtab,
    DirectQlen,
    Rate64,
    Ceil64,
    Pad,
    Max,
}

impl From<u16> for TcaHtb {
    fn from(v: u16) -> Self {
        match v {
            0 => TcaHtb::Unspec,
            1 => TcaHtb::Parms,
            2 => TcaHtb::Init,
            3 => TcaHtb::Ctab,
            4 => TcaHtb::Rtab,
            5 => TcaHtb::DirectQlen,
            6 => TcaHtb::Rate64,
            7 => TcaHtb::Ceil64,
            8 => TcaHtb::Pad,
            _ => TcaHtb::Max,
        }
    }
}

impl Htb {
    pub fn new(opts: Vec<TcOption>) -> Self {
        unmarshal_htb(opts)
    }
}

impl HtbXstats {
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        unmarshal_htb_xstats(bytes)
    }
}

fn unmarshal_htb(opts: Vec<TcOption>) -> Htb {
    let mut htb = Htb::default();

    for opt in opts {
        let kind = TcaHtb::from(opt.kind);
        match kind {
            TcaHtb::Parms => htb.parms = unmarshal_htb_opt(opt.bytes.as_slice()).ok(),
            TcaHtb::Init => htb.init = unmarshal_htb_glob(opt.bytes.as_slice()).ok(),
            TcaHtb::Ctab => htb.ctab = opt.bytes,
            TcaHtb::Rtab => htb.rtab = opt.bytes,
            TcaHtb::DirectQlen => {
                htb.direct_qlen = {
                    if opt.bytes.len() < 4 {
                        // TODO: log error
                        None
                    } else {
                        Some(u32::from_ne_bytes(opt.bytes[0..4].try_into().unwrap()))
                    }
                }
            }
            TcaHtb::Rate64 => {
                htb.rate64 = {
                    if opt.bytes.len() < 8 {
                        // TODO: log error
                        None
                    } else {
                        Some(u64::from_ne_bytes(opt.bytes[0..8].try_into().unwrap()))
                    }
                }
            }
            TcaHtb::Ceil64 => {
                htb.ceil64 = {
                    if opt.bytes.len() < 8 {
                        // TODO: log error
                        None
                    } else {
                        Some(u64::from_ne_bytes(opt.bytes[0..8].try_into().unwrap()))
                    }
                }
            }
            _ => (),
        }
    }

    htb
}

fn unmarshal_htb_opt(bytes: &[u8]) -> Result<HtbOpt, Error> {
    bincode::deserialize(bytes).map_err(Error::UnmarshalStruct)
}

fn unmarshal_htb_glob(bytes: &[u8]) -> Result<HtbGlob, Error> {
    bincode::deserialize(bytes).map_err(Error::UnmarshalStruct)
}

fn unmarshal_htb_xstats(bytes: &[u8]) -> Result<HtbXstats, Error> {
    bincode::deserialize(bytes).map_err(Error::UnmarshalStruct)
}
