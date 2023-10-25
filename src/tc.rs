use crate::{
    class::htb::Htb,
    constants::*,
    errors::TcError,
    link, netlink,
    qdiscs::{
        clsact::Clsact,
        fq_codel::{FqCodel, FqCodelXStats},
    },
    types::*,
    HtbXstats,
};

/// `qdiscs` returns a list of all qdiscs on the system.
/// The underlying implementation makes a netlink call with the `RTM_GETQDISC` command.
pub fn qdiscs<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, TcError> {
    let mut tcs = Vec::new();

    let messages = T::new()?.qdiscs()?;
    for message in &messages {
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
                TcAttr::Stats(bytes) => attribute.stats = parse_stats(&bytes).ok(),
                TcAttr::Xstats(bytes) => xstats.extend(bytes.as_slice()),
                TcAttr::Stats2(stats) => attribute.stats2 = parse_stats2(&stats).ok(),
                _ => (),

            }
        }

        attribute.qdisc = parse_qdiscs(attribute.kind.as_str(), options);
        attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice()).ok();

        tcs.push(Tc { msg: tc, attr: attribute });
    }

    Ok(tcs)
}

/// `class_for_index` returns a list of all classes for a given interface index.
/// The underlying implementation makes a netlink call with the `RTM_GETCLASS` command.
pub fn class_for_index<T: netlink::NetlinkConnection>(index: u32) -> Result<Vec<Tc>, TcError> {
    let mut tcs = Vec::new();

    let messages = T::new()?.classes(index as i32)?;
    for message in &messages {
        let tc = TcMessage {
            index,
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
                TcAttr::Stats(bytes) => attribute.stats = parse_stats(&bytes).ok(),
                TcAttr::Xstats(bytes) => xstats.extend(bytes.as_slice()),
                TcAttr::Stats2(stats) => attribute.stats2 = parse_stats2(stats).ok(),
                _ => (),
            }
        }

        attribute.class = parse_classes(attribute.kind.as_str(), opts);
        attribute.xstats = parse_xstats(attribute.kind.as_str(), xstats.as_slice()).ok();

        tcs.push(Tc { msg: tc, attr: attribute });
    }

    Ok(tcs)
}

/// `class` returns a list of all classes for a given interface name.
/// It retrieves the list of links and then calls `class_for_index`
/// for the link with the matching name.
pub fn class<T: netlink::NetlinkConnection>(name: &str) -> Result<Vec<Tc>, TcError> {
    let links = link::links::<T>()?;

    if let Some(link) = links.iter().find(|link| link.name == name) {
        class_for_index::<T>(link.index)
    } else {
        Ok(Vec::new())
    }
}

/// `classes` returns a list of all classes on the system.
/// It retrieves the list of links and then calls `classes` for each link.
pub fn classes<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, TcError> {
    let mut tcs = Vec::new();

    let links = link::links::<T>()?;
    for link in links {
        tcs.append(&mut class_for_index::<T>(link.index)?);
    }

    Ok(tcs)
}

pub fn tc_stats<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, TcError> {
    let mut tcs = qdiscs::<T>()?;
    tcs.append(&mut classes::<T>()?);

    Ok(tcs)
}


fn parse_stats(bytes: &[u8]) -> Result<Stats, TcError> {
    bincode::deserialize(bytes).map_err(|e| TcError::UnmarshalStruct(e))
}

fn parse_stats2(stats2: &Vec<TcStats2>) -> Result<Stats2, TcError> {
    let mut stats = Stats2::default();
    let mut errors = Vec::new();
    for stat in stats2 {
        match stat {
            TcStats2::StatsBasic(bytes) => {
                match bincode::deserialize(bytes.as_slice()) {
                    Ok(stats_basic) => stats.basic = Some(stats_basic),
                    Err(e) => errors.push(format!("Failed to parse StatsBasic: {e}")),
                }
            }
            TcStats2::StatsQueue(bytes) => {
                match bincode::deserialize(bytes.as_slice()) {
                    Ok(stats_queue) => stats.queue = Some(stats_queue),
                    Err(e) => errors.push(format!("Failed to parse StatsQueue: {e}")),
                }
            }
            // TcStats2::StatsApp(bytes) => stats.app = bincode::deserialize(bytes.as_slice()).ok(),
            _ => (),
        }
    }

    if errors.len() > 0 {
        let message = errors.join(", ");
        Err(TcError::UnmarshalStructs(message))
    } else {
        Ok(stats)
    }
}

fn parse_qdiscs(kind: &str, opts: Vec<TcOption>) -> Option<QDisc> {
    match kind {
        FQ_CODEL => Some(QDisc::FqCodel(FqCodel::new(opts))),
        CLSACT => Some(QDisc::Clsact(Clsact {})),
        HTB => Htb::new(opts).init.and_then(|htb| Some(QDisc::Htb(htb))),
        _ => None,
    }
}

fn parse_classes(kind: &str, opts: Vec<TcOption>) -> Option<Class> {
    match kind {
        HTB => Some(Class::Htb(Htb::new(opts))),
        _ => None,
    }
}

fn parse_xstats(kind: &str, bytes: &[u8]) -> Result<XStats, TcError> {
    match kind {
        FQ_CODEL => FqCodelXStats::new(&bytes)
            .and_then(|x| Ok(XStats::FqCodel(x))),
        HTB => HtbXstats::new(&bytes).and_then(|x| Ok(XStats::Htb(x))),
        _ => Err(TcError::UnimplementedAttribute(format!("XStats for {kind}"))),
    }
}
