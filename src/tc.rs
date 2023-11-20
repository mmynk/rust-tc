use crate::class::{Htb, HtbXstats};
use crate::constants::{CLSACT, FQ_CODEL, HTB};
use crate::errors::Error;
use crate::qdiscs::{Clsact, FqCodel, FqCodelXStats};
use crate::types::{
    Attribute, Class, QDisc, Stats, Stats2, Tc, TcAttr, TcMessage, TcMsg, TcOption, TcStats2,
    XStats,
};
use crate::{OpenOptions, RtNetlinkMessage};

/// `qdiscs` returns a list of queuing disciplines by parsing the passed `TcMsg` vector.
pub fn qdiscs(message: TcMsg, opts: &OpenOptions) -> Result<Tc, Error> {
    let tc = TcMessage {
        index: message.header.index as u32,
        handle: message.header.handle,
        parent: message.header.parent,
    };
    let mut attribute = Attribute::default();

    let mut tc_opts = Vec::new();
    let mut xstats = Vec::new();
    for attr in &message.attrs {
        match attr {
            TcAttr::Kind(kind) => attribute.kind = kind.to_string(),
            TcAttr::Options(options) => tc_opts = options.to_vec(),
            TcAttr::Stats(bytes) => attribute.stats = parse_stats(bytes).ok(),
            TcAttr::Xstats(bytes) => xstats.extend(bytes.as_slice()),
            TcAttr::Stats2(stats) => attribute.stats2 = parse_stats2(stats).ok(),
            _ => {
                if opts.fail_on_unknown_attribute {
                    return Err(Error::UnimplementedAttribute(format!(
                        "Attribute {:?} not implemented",
                        attr
                    )));
                }
            }
        }
    }

    attribute.qdisc = parse_qdiscs(attribute.kind.as_str(), tc_opts, opts)?;
    attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice(), opts)?;

    Ok(Tc {
        msg: tc,
        attr: attribute,
    })
}

/// `classes` returns a list of traffic control classes by parsing the passed `TcMsg` vector.
pub fn classes(message: TcMsg, opts: &OpenOptions) -> Result<Tc, Error> {
    let tc = TcMessage {
        index: message.header.index as u32,
        handle: message.header.handle,
        parent: message.header.parent,
    };
    let mut attribute = Attribute::default();

    let mut tc_opts = vec![];
    let mut xstats = Vec::new();
    for attr in &message.attrs {
        match attr {
            TcAttr::Kind(kind) => attribute.kind = kind.to_string(),
            TcAttr::Options(options) => tc_opts = options.to_vec(),
            TcAttr::Stats(bytes) => attribute.stats = parse_stats(bytes).ok(),
            TcAttr::Xstats(bytes) => xstats.extend(bytes.as_slice()),
            TcAttr::Stats2(stats) => attribute.stats2 = parse_stats2(stats).ok(),
            _ => {
                if opts.fail_on_unknown_attribute {
                    return Err(Error::UnimplementedAttribute(format!(
                        "Attribute {:?} not implemented",
                        attr
                    )));
                }
            }
        }
    }

    attribute.class = parse_classes(attribute.kind.as_str(), tc_opts, opts)?;
    attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice(), opts)?;

    Ok(Tc {
        msg: tc,
        attr: attribute,
    })
}

pub fn tc_stats(messages: Vec<RtNetlinkMessage>, opts: &OpenOptions) -> Result<Vec<Tc>, Error> {
    let mut tcs = Vec::with_capacity(messages.len());

    for message in messages {
        match message {
            RtNetlinkMessage::GetQdisc(message) => tcs.push(qdiscs(message, opts)?),
            RtNetlinkMessage::GetClass(message) => tcs.push(classes(message, opts)?),
            _ => {}
        }
    }

    Ok(tcs)
}

fn parse_stats(bytes: &[u8]) -> Result<Stats, Error> {
    bincode::deserialize(bytes).map_err(Error::UnmarshalStruct)
}

fn parse_stats2(stats2: &Vec<TcStats2>) -> Result<Stats2, Error> {
    let mut stats = Stats2::default();
    let mut errors = Vec::new();
    for stat in stats2 {
        match stat {
            TcStats2::StatsBasic(bytes) => match bincode::deserialize(bytes.as_slice()) {
                Ok(stats_basic) => stats.basic = Some(stats_basic),
                Err(e) => errors.push(format!("Failed to parse StatsBasic: {e}")),
            },
            TcStats2::StatsQueue(bytes) => match bincode::deserialize(bytes.as_slice()) {
                Ok(stats_queue) => stats.queue = Some(stats_queue),
                Err(e) => errors.push(format!("Failed to parse StatsQueue: {e}")),
            },
            // TODO: StatsApp parsing
            // TcStats2::StatsApp(bytes) => stats.app = bincode::deserialize(bytes.as_slice()).ok(),
            _ => (),
        }
    }

    if !errors.is_empty() {
        let message = errors.join(", ");
        Err(Error::Parse(format!(
            "Failed to unmarshal structs: {message}"
        )))
    } else {
        Ok(stats)
    }
}

fn parse_qdiscs(
    kind: &str,
    tc_opts: Vec<TcOption>,
    opts: &OpenOptions,
) -> Result<Option<QDisc>, Error> {
    let qdisc = match kind {
        FQ_CODEL => Some(QDisc::FqCodel(FqCodel::new(tc_opts))),
        CLSACT => Some(QDisc::Clsact(Clsact {})),
        HTB => Htb::new(tc_opts).init.map(QDisc::Htb),
        _ => {
            if opts.fail_on_unknown_option {
                return Err(Error::UnimplementedAttribute(format!(
                    "QDisc {kind} not implemented",
                )));
            } else {
                None
            }
        }
    };
    Ok(qdisc)
}

fn parse_classes(
    kind: &str,
    tc_opts: Vec<TcOption>,
    opts: &OpenOptions,
) -> Result<Option<Class>, Error> {
    let class = match kind {
        HTB => Some(Class::Htb(Htb::new(tc_opts))),
        _ => {
            if opts.fail_on_unknown_option {
                return Err(Error::UnimplementedAttribute(format!(
                    "Class {kind} not implemented",
                )));
            } else {
                None
            }
        }
    };
    Ok(class)
}

fn parse_xstats(kind: &str, bytes: &[u8], opts: &OpenOptions) -> Result<Option<XStats>, Error> {
    let xstats = match kind {
        FQ_CODEL => FqCodelXStats::new(bytes).ok().map(XStats::FqCodel),
        HTB => HtbXstats::new(bytes).ok().map(XStats::Htb),
        _ => {
            if opts.fail_on_unknown_option {
                return Err(Error::UnimplementedAttribute(format!(
                    "XStats {kind} not implemented",
                )));
            } else {
                None
            }
        }
    };
    Ok(xstats)
}
