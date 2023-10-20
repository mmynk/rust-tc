use netlink_packet_route::tc as netlink_tc;
use netlink_packet_utils::{nla, Parseable};

use crate::{
    class::htb::Htb,
    constants::*,
    errors::TcError,
    netlink,
    qdiscs::{
        clsact::Clsact,
        fq_codel::{FqCodel, FqCodelXStats},
    },
    link,
    types::*,
    HtbXstats,
};

fn parse_stats(attr: &mut Attribute, tc_stats: &netlink_tc::Stats) {
    let stats = attr.stats.get_or_insert(Stats::default());
    stats.bytes = tc_stats.bytes;
    stats.packets = tc_stats.packets;
    stats.drops = tc_stats.drops;
    stats.overlimits = tc_stats.overlimits;
    stats.bps = tc_stats.bps;
    stats.pps = tc_stats.pps;
    stats.qlen = tc_stats.qlen;
    stats.backlog = tc_stats.backlog;
}

fn parse_stats_basic(attr: &mut Attribute, bytes: &Vec<u8>) {
    let stats_basic =
        netlink_tc::nlas::StatsBasic::parse(&netlink_tc::StatsBasicBuffer::new(bytes));
    if let Ok(stats_basic) = stats_basic {
        let stats = attr.stats2.get_or_insert(Stats2::default());
        stats.basic = Some(StatsBasic {
            bytes: stats_basic.bytes,
            packets: stats_basic.packets,
        });
    }
}

fn parse_stats_queue(attr: &mut Attribute, bytes: &Vec<u8>) {
    let stats_queue =
        netlink_tc::nlas::StatsQueue::parse(&netlink_tc::StatsQueueBuffer::new(bytes));
    if let Ok(stats_queue) = stats_queue {
        let stats = attr.stats2.get_or_insert(Stats2::default());
        stats.queue = Some(StatsQueue {
            qlen: stats_queue.qlen,
            backlog: stats_queue.backlog,
            drops: stats_queue.drops,
            requeues: stats_queue.requeues,
            overlimits: stats_queue.overlimits,
        });
    }
}

fn parse_qdiscs(attr: &mut Attribute, opts: Vec<&nla::DefaultNla>) -> Result<(), TcError> {
    if let Some(kind) = &attr.kind {
        let kind = kind.as_str();
        match kind {
            FQ_CODEL => attr.qdisc = Some(QDisc::FqCodel(FqCodel::new(opts))),
            CLSACT => attr.qdisc = Some(QDisc::Clsact(Clsact {})),
            HTB => {
                attr.qdisc = {
                    let htb = Htb::new(opts)?;
                    match htb.init {
                        Some(init) => Some(QDisc::Htb(init)),
                        None => None,
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

fn parse_classes(attr: &mut Attribute, opts: Vec<&nla::DefaultNla>) -> Result<(), TcError> {
    if let Some(kind) = &attr.kind {
        let kind = kind.as_str();
        attr.class = match kind {
            HTB => {
                let htb = Htb::new(opts)?;
                Some(Class::Htb(htb))
            }
            _ => None,
        }
    }
    Ok(())
}

fn parse_xstats(attr: &mut Attribute, bytes: Vec<u8>) -> Result<(), TcError> {
    if let Some(kind) = &attr.kind {
        let kind = kind.as_str();
        attr.xstats = match kind {
            FQ_CODEL => FqCodelXStats::new(&bytes)
                .and_then(|x| Ok(XStats::FqCodel(x)))
                .ok(),
            HTB => HtbXstats::new(&bytes).and_then(|x| Ok(XStats::Htb(x))).ok(),
            _ => None,
        }
    }
    Ok(())
}

/// `qdiscs` returns a list of all qdiscs on the system.
/// The underlying implementation makes a netlink call with the `RTM_GETQDISC` command.
pub fn qdiscs<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, TcError> {
    let mut tcs = Vec::new();

    let messages = T::new()?.qdiscs()?;
    // println!("messages: {:?}", messages);
    for message in &messages {
        let tc = TcMessage {
            index: message.header.index as u32,
            handle: message.header.handle,
            parent: message.header.parent,
        };
        let mut attr = Attribute::default();

        let mut opts: Vec<&nla::DefaultNla> = vec![];
        let mut xstats = Vec::new();
        for nla in &message.nlas {
            match nla {
                netlink_tc::Nla::Kind(kind) => attr.kind = Some(kind.clone()),
                netlink_tc::Nla::Stats2(stats) => {
                    for stat in stats {
                        match stat {
                            netlink_tc::Stats2::StatsBasic(stat) => {
                                parse_stats_basic(&mut attr, stat)
                            }
                            netlink_tc::Stats2::StatsQueue(stat) => {
                                parse_stats_queue(&mut attr, stat)
                            }
                            // TODO: parse Stats2::StatsApp
                            _ => (),
                        }
                    }
                }
                netlink_tc::Nla::Stats(stats) => parse_stats(&mut attr, stats),
                netlink_tc::Nla::Options(tc_opts) => {
                    for opt in tc_opts {
                        if let netlink_tc::TcOpt::Other(opt) = opt {
                            opts.push(opt);
                        }
                    }
                }
                netlink_tc::Nla::XStats(bytes) => xstats = bytes.clone(),
                _ => (),
            }
        }

        parse_qdiscs(&mut attr, opts)?;
        parse_xstats(&mut attr, xstats)?;

        tcs.push(Tc { msg: tc, attr });
    }

    Ok(tcs)
}

/// `class` returns a list of all classes for a given interface.
/// The underlying implementation makes a netlink call with the `RTM_GETCLASS` command.
pub fn class<T: netlink::NetlinkConnection>(index: u32) -> Result<Vec<Tc>, TcError> {
    let mut tcs = Vec::new();

    let messages = T::new()?.classes(index as i32)?;
    for message in &messages {
        let tc = TcMessage {
            index,
            handle: message.header.handle,
            parent: message.header.parent,
        };
        let mut attr = Attribute::default();

        let mut opts: Vec<&nla::DefaultNla> = vec![];
        let mut xstats = Vec::new();
        for nla in &message.nlas {
            match nla {
                netlink_tc::Nla::Kind(kind) => attr.kind = Some(kind.clone()),
                netlink_tc::Nla::Stats2(stats) => {
                    for stat in stats {
                        match stat {
                            netlink_tc::Stats2::StatsBasic(stat) => {
                                parse_stats_basic(&mut attr, stat)
                            }
                            netlink_tc::Stats2::StatsQueue(stat) => {
                                parse_stats_queue(&mut attr, stat)
                            }
                            _ => (),
                        }
                    }
                }
                netlink_tc::Nla::Stats(stats) => parse_stats(&mut attr, stats),
                netlink_tc::Nla::Options(tc_opts) => {
                    for opt in tc_opts {
                        if let netlink_tc::TcOpt::Other(opt) = opt {
                            opts.push(opt);
                        }
                    }
                }
                netlink_tc::Nla::XStats(bytes) => xstats = bytes.clone(),
                _ => (),
            }
        }

        parse_classes(&mut attr, opts)?;
        parse_xstats(&mut attr, xstats)?;

        tcs.push(Tc { msg: tc, attr });
    }

    Ok(tcs)
}

/// `classes` returns a list of all classes on the system.
/// It retrieves the list of links and then calls `classes` for each link.
pub fn classes<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, TcError> {
    let mut tcs = Vec::new();

    let links = link::links::<T>()?;
    for link in links {
        tcs.append(&mut class::<T>(link.index)?);
    }

    Ok(tcs)
}

pub fn tc_stats<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, TcError> {
    let mut tcs = qdiscs::<T>()?;
    tcs.append(&mut classes::<T>()?);

    Ok(tcs)
}
