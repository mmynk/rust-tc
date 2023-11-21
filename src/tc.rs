use crate::class::{Htb, HtbXstats};
use crate::constants::{CLSACT, FQ_CODEL, HTB};
use crate::errors::TcError;
use crate::qdiscs::{Clsact, FqCodel, FqCodelXStats};
use crate::types::{
    Attribute, Class, QDisc, Stats, Stats2, Tc, TcAttr, TcMessage, TcMsg, TcOption, TcStats2,
    XStats,
};
use crate::{ParseOptions, RtNetlinkMessage};

fn get_qdiscs(message: TcMsg, classful: bool, opts: &ParseOptions) -> Result<Tc, TcError> {
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
                    return Err(TcError::Parse(format!(
                        "Attribute {:?} not implemented",
                        attr
                    )));
                }
            }
        }
    }

    if classful {
        attribute.class = parse_classes(attribute.kind.as_str(), tc_opts, opts)?;
    } else {
        attribute.qdisc = parse_qdiscs(attribute.kind.as_str(), tc_opts, opts)?;
    }
    attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice(), opts)?;

    Ok(Tc {
        msg: tc,
        attr: attribute,
    })
}

/// `qdiscs` returns a list of queuing disciplines by parsing the passed `TcMsg` vector.
pub fn qdiscs(message: TcMsg, opts: &ParseOptions) -> Result<Tc, TcError> {
    get_qdiscs(message, false, opts)
}

/// `classes` returns a list of traffic control classes by parsing the passed `TcMsg` vector.
pub fn classes(message: TcMsg, opts: &ParseOptions) -> Result<Tc, TcError> {
    get_qdiscs(message, true, opts)
}

pub fn tc_stats(messages: Vec<RtNetlinkMessage>, opts: &ParseOptions) -> Result<Vec<Tc>, TcError> {
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

fn parse_stats(bytes: &[u8]) -> Result<Stats, TcError> {
    bincode::deserialize(bytes).map_err(|e| TcError::Parse(e.to_string()))
}

fn parse_stats2(stats2: &Vec<TcStats2>) -> Result<Stats2, TcError> {
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
        Err(TcError::Parse(format!(
            "Failed to unmarshal structs: {message}"
        )))
    } else {
        Ok(stats)
    }
}

fn parse_qdiscs(
    kind: &str,
    tc_opts: Vec<TcOption>,
    opts: &ParseOptions,
) -> Result<Option<QDisc>, TcError> {
    let qdisc = match kind {
        FQ_CODEL => Some(QDisc::FqCodel(FqCodel::new(tc_opts))),
        CLSACT => Some(QDisc::Clsact(Clsact {})),
        HTB => Htb::new(tc_opts).init.map(QDisc::Htb),
        _ => {
            if opts.fail_on_unknown_option {
                return Err(TcError::Parse(format!(
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
    opts: &ParseOptions,
) -> Result<Option<Class>, TcError> {
    let class = match kind {
        HTB => Some(Class::Htb(Htb::new(tc_opts))),
        _ => {
            if opts.fail_on_unknown_option {
                return Err(TcError::Parse(format!(
                    "Class {kind} not implemented",
                )));
            } else {
                None
            }
        }
    };
    Ok(class)
}

fn parse_xstats(kind: &str, bytes: &[u8], opts: &ParseOptions) -> Result<Option<XStats>, TcError> {
    let xstats = match kind {
        FQ_CODEL => FqCodelXStats::new(bytes).ok().map(XStats::FqCodel),
        HTB => HtbXstats::new(bytes).ok().map(XStats::Htb),
        _ => {
            if opts.fail_on_unknown_option {
                return Err(TcError::Parse(format!(
                    "XStats {kind} not implemented",
                )));
            } else {
                None
            }
        }
    };
    Ok(xstats)
}
