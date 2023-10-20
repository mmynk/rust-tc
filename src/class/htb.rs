use netlink_packet_utils::nla::{self, Nla};
use serde::{Deserialize, Serialize};

use crate::constants::ATTR_LEN;
use crate::errors::TcError;
use crate::RateSpec;

/// The Hierarchy Token Bucket implements a rich linksharing hierarchy of classes
/// with an emphasis on conforming to existing practices.
/// HTB facilitates guaranteeing bandwidth to classes,
/// while also allowing specification of upper limits to inter-class sharing.
/// It contains shaping elements, based on TBF and can prioritize classes.
///
/// Defined in `include/uapi/linux/pkt_sched.h`.
#[derive(Default, Debug, PartialEq)]
pub struct Htb {
    pub parms: Option<HtbOpt>,
    pub init: Option<HtbGlob>,
    pub ctab: Vec<u8>,
    pub rtab: Vec<u8>,
    pub direct_qlen: u32,
    pub rate64: u64,
    pub ceil64: u64,
}

const HTB_OPT_LEN: usize = 44;

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

const HTB_GLOB_LEN: usize = 4 * 5;

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
    pub fn new(nla: Vec<&nla::DefaultNla>) -> Result<Self, TcError> {
        unmarshal_htb(nla)
    }
}

impl HtbXstats {
    pub fn new(buf: &[u8]) -> Result<Self, TcError> {
        let result = unmarshal_htb_xstats(buf);
        result
    }
}

fn unmarshal_htb(nlas: Vec<&nla::DefaultNla>) -> Result<Htb, TcError> {
    let mut htb = Htb::default();

    for nla in nlas {
        let kind = TcaHtb::from(nla.kind());
        match kind {
            TcaHtb::Parms => {
                htb.parms = {
                    let mut buf = [0u8; HTB_OPT_LEN];
                    nla.emit_value(&mut buf);
                    Some(unmarshal_htb_opt(&buf)?)
                }
            }
            TcaHtb::Init => {
                htb.init = {
                    let mut buf = [0u8; HTB_GLOB_LEN];
                    nla.emit_value(&mut buf);
                    Some(unmarshal_htb_glob(&buf)?)
                }
            }
            TcaHtb::Ctab => {
                htb.ctab = {
                    let mut buf = vec![0u8; nla.value_len()];
                    nla.emit_value(&mut buf);
                    buf.to_vec()
                }
            }
            TcaHtb::Rtab => {
                htb.rtab = {
                    let mut buf = vec![0u8; nla.value_len()];
                    nla.emit_value(&mut buf);
                    buf.to_vec()
                }
            }
            TcaHtb::DirectQlen => {
                htb.direct_qlen = {
                    let mut buf = [0u8; ATTR_LEN];
                    nla.emit_value(&mut buf);
                    u32::from_ne_bytes(buf)
                }
            }
            TcaHtb::Rate64 => {
                htb.rate64 = {
                    let mut buf = [0u8; 8];
                    nla.emit_value(&mut buf);
                    u64::from_ne_bytes(buf)
                }
            }
            TcaHtb::Ceil64 => {
                htb.ceil64 = {
                    let mut buf = [0u8; 8];
                    nla.emit_value(&mut buf);
                    u64::from_ne_bytes(buf)
                }
            }
            _ => (),
        }
    }

    Ok(htb)
}

fn unmarshal_htb_opt(buf: &[u8]) -> Result<HtbOpt, TcError> {
    bincode::deserialize(buf).map_err(|e| TcError::UnmarshalStruct(e))
}

fn unmarshal_htb_glob(buf: &[u8]) -> Result<HtbGlob, TcError> {
    bincode::deserialize(buf).map_err(|e| TcError::UnmarshalStruct(e))
}

fn unmarshal_htb_xstats(buf: &[u8]) -> Result<HtbXstats, TcError> {
    bincode::deserialize(buf).map_err(|e| TcError::UnmarshalStruct(e))
}
