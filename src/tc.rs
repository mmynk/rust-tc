use std::collections::BTreeMap;

use netlink_packet_route::tc as netlink_tc;
use netlink_packet_utils::{nla, Parseable};

use crate::{
    constants::FQ_CODEL,
    errors::TcError,
    netlink,
    qdiscs::fq_codel::{FqCodel, FqCodelXStats},
};

#[derive(Clone, Debug, Default)]
pub struct Stats {
    // Stats2::StatsBasic
    bytes: u64,
    packets: u32,

    // Stats2::StatsQueue
    qlen: u32,
    backlog: u32,
    drops: u32,
    requeues: u32,
    overlimits: u32,

    // XStats
    xstats: Option<XStats>,

    bps: u32,
    pps: u32,
}

#[derive(Clone, Debug)]
pub enum QDisc {
    FqCodel(FqCodel),
}

#[derive(Clone, Debug)]
pub enum XStats {
    FqCodel(FqCodelXStats),
}

#[derive(Clone, Debug, Default)]
pub struct Tc {
    handle: Option<u32>,
    parent: Option<u32>,
    kind: Option<String>,
    stats: Option<Stats>,
    // backlog: Option<Backlog>,
    qdisc: Option<QDisc>,
}

fn parse_stats(tc: &mut Tc, tc_stats: &netlink_tc::Stats) {
    let stats = tc.stats.get_or_insert(Stats::default());
    stats.bps = tc_stats.bps;
    stats.pps = tc_stats.pps;
}

fn parse_stats_basic(tc: &mut Tc, bytes: &Vec<u8>) {
    let stats_basic =
        netlink_tc::nlas::StatsBasic::parse(&netlink_tc::StatsBasicBuffer::new(bytes));
    if let Ok(stats_basic) = stats_basic {
        let stats = tc.stats.get_or_insert(Stats::default());
        stats.bytes = stats_basic.bytes;
        stats.packets = stats_basic.packets;
    }
}

fn parse_stats_queue(tc: &mut Tc, bytes: &Vec<u8>) {
    let stats_queue =
        netlink_tc::nlas::StatsQueue::parse(&netlink_tc::StatsQueueBuffer::new(bytes));
    if let Ok(stats_queue) = stats_queue {
        let stats = tc.stats.get_or_insert(Stats::default());
        stats.qlen = stats_queue.qlen;
        stats.backlog = stats_queue.backlog;
        stats.drops = stats_queue.drops;
        stats.requeues = stats_queue.requeues;
        stats.overlimits = stats_queue.overlimits;
    }
}

fn parse_qdiscs(tc: &mut Tc, opts: Vec<&nla::DefaultNla>) {
    if let Some(kind) = &tc.kind {
        let kind = kind.as_str();
        match kind {
            FQ_CODEL => tc.qdisc = Some(QDisc::FqCodel(FqCodel::new(opts))),
            _ => (),
        }
    }
}

fn parse_xstats(tc: &mut Tc, bytes: Vec<u8>) {
    if bytes.len() < 40 {
        return;
    }
    let stats = tc.stats.get_or_insert(Stats::default());
    if let Some(kind) = &tc.kind {
        let bytes: [u8; 40] = bytes[..40].try_into().unwrap();
        let kind = kind.as_str();
        match kind {
            FQ_CODEL => stats.xstats = Some(XStats::FqCodel(FqCodelXStats::new(bytes))),
            _ => (),
        }
    }
}

pub fn tc_stats() -> Result<BTreeMap<u32, Vec<Tc>>, TcError> {
    let messages = netlink::get_qdiscs()?;
    let mut tc_map = BTreeMap::new();

    for message in &messages {
        let mut tc = Tc::default();
        let mut opts: Vec<&nla::DefaultNla> = vec![];
        let mut xstats = Vec::new();

        for nla in &message.nlas {
            match nla {
                netlink_tc::Nla::Kind(kind) => tc.kind = Some(kind.clone()),
                netlink_tc::Nla::Stats2(stats) => {
                    for stat in stats {
                        match stat {
                            netlink_tc::Stats2::StatsBasic(stat) => {
                                parse_stats_basic(&mut tc, stat)
                            }
                            netlink_tc::Stats2::StatsQueue(stat) => {
                                parse_stats_queue(&mut tc, stat)
                            }
                            // TODO: parse Stats2::StatsApp
                            _ => (),
                        }
                    }
                }
                netlink_tc::Nla::Stats(stats) => parse_stats(&mut tc, stats),
                netlink_tc::Nla::Options(tc_opts) => {
                    for opt in tc_opts {
                        if let netlink_tc::TcOpt::Other(opt) = opt {
                            opts.push(opt);
                        }
                    }
                }
                // netlink_tc::Nla::QDisc(q_disc) => parse_qdisc(&mut tc, q_disc),
                netlink_tc::Nla::XStats(bytes) => xstats = bytes.clone(),
                _ => (),
            }
        }

        parse_qdiscs(&mut tc, opts);
        parse_xstats(&mut tc, xstats);

        let dev = message.header.index as u32;
        tc_map.entry(dev).or_insert(Vec::new()).push(tc);
    }

    Ok(tc_map)
}
