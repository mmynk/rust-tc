use crate::class::{Htb, HtbXstats};
use crate::constants::{CLSACT, FQ_CODEL, HTB};
use crate::errors::Error;
use crate::qdiscs::{Clsact, FqCodel, FqCodelXStats};
use crate::RtNetlinkMessage;
use crate::types::{Attribute, Class, QDisc, Stats, Stats2, Tc, TcAttr, TcMessage, TcMsg, TcOption, TcStats2, XStats};

/// `qdiscs` returns a list of queuing disciplines by parsing the passed `TcMsg` vector.
pub fn qdiscs(message: TcMsg) -> Result<Tc, Error> {
    let tc = TcMessage {
        index: message.header.index as u32,
        handle: message.header.handle,
        parent: message.header.parent,
    };
    let mut attribute = Attribute::default();

    let mut options = Vec::new();
    let mut xstats = Vec::new();
    for attr in &message.attrs {
        match attr {
            TcAttr::Kind(kind) => attribute.kind = kind.to_string(),
            TcAttr::Options(opts) => options = opts.to_vec(),
            TcAttr::Stats(bytes) => attribute.stats = parse_stats(bytes).ok(),
            TcAttr::Xstats(bytes) => xstats.extend(bytes.as_slice()),
            TcAttr::Stats2(stats) => attribute.stats2 = parse_stats2(stats).ok(),
            _ => (),
        }
    }

    attribute.qdisc = parse_qdiscs(attribute.kind.as_str(), options);
    attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice()).ok();

    Ok(Tc {
        msg: tc,
        attr: attribute,
    })
}

/// `classes` returns a list of traffic control classes by parsing the passed `TcMsg` vector.
pub fn classes(message: TcMsg) -> Result<Tc, Error> {
    let tc = TcMessage {
        index: message.header.index as u32,
        handle: message.header.handle,
        parent: message.header.parent,
    };
    let mut attribute = Attribute::default();

    let mut opts = vec![];
    let mut xstats = Vec::new();
    for attr in &message.attrs {
        match attr {
            TcAttr::Kind(kind) => attribute.kind = kind.to_string(),
            TcAttr::Options(tc_opts) => opts = tc_opts.to_vec(),
            TcAttr::Stats(bytes) => attribute.stats = parse_stats(bytes).ok(),
            TcAttr::Xstats(bytes) => xstats.extend(bytes.as_slice()),
            TcAttr::Stats2(stats) => attribute.stats2 = parse_stats2(stats).ok(),
            _ => (),
        }
    }

    attribute.class = parse_classes(attribute.kind.as_str(), opts);
    attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice()).ok();

    Ok(Tc {
        msg: tc,
        attr: attribute,
    })
}

pub fn tc_stats(messages: Vec<RtNetlinkMessage>) -> Result<Vec<Tc>, Error> {
    let mut tcs = Vec::with_capacity(messages.len());

    for message in messages {
        match message {
            RtNetlinkMessage::GetQdisc(message) => tcs.push(qdiscs(message)?),
            RtNetlinkMessage::GetClass(message) => tcs.push(classes(message)?),
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
        Err(Error::Parse(format!("Failed to unmarshal structs: {message}")))
    } else {
        Ok(stats)
    }
}

fn parse_qdiscs(kind: &str, opts: Vec<TcOption>) -> Option<QDisc> {
    match kind {
        FQ_CODEL => Some(QDisc::FqCodel(FqCodel::new(opts))),
        CLSACT => Some(QDisc::Clsact(Clsact {})),
        HTB => Htb::new(opts).init.map(QDisc::Htb),
        _ => None,
    }
}

fn parse_classes(kind: &str, opts: Vec<TcOption>) -> Option<Class> {
    match kind {
        HTB => Some(Class::Htb(Htb::new(opts))),
        _ => None,
    }
}

fn parse_xstats(kind: &str, bytes: &[u8]) -> Result<XStats, Error> {
    match kind {
        FQ_CODEL => FqCodelXStats::new(bytes).map(XStats::FqCodel),
        HTB => HtbXstats::new(bytes).map(XStats::Htb),
        _ => Err(Error::UnimplementedAttribute(format!(
            "XStats for {kind}"
        ))),
    }
}
