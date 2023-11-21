use netlink_packet_core::NetlinkHeader;
use netlink_packet_route::TcMessage;

use crate::class::{Htb, HtbGlob, HtbOpt, HtbXstats};
use crate::qdiscs::{FqCodel, FqCodelXStats};
use crate::test_data::{get_classes, get_links, get_qdiscs, nlas, qdisc};
use crate::types::{Class, QDisc, RateSpec, XStats};

use super::*;

#[test]
fn test_no_queue() {
    let messages = vec![get_qdiscs()[0].clone()];
    let stats = OpenOptions::new().tc(messages).unwrap();

    let tc = stats.get(0).unwrap();
    // message
    assert_eq!(tc.msg.index, 1);
    assert_eq!(tc.msg.handle, 0);
    assert_eq!(tc.msg.parent, 4294967295);
    // attr
    assert_eq!(tc.attr.kind.as_str(), "noqueue");
    let basic = tc.attr.stats2.as_ref().unwrap().basic.as_ref().unwrap();
    assert_eq!(basic.bytes, 0);
    assert_eq!(basic.packets, 0);
    let queue = tc.attr.stats2.as_ref().unwrap().queue.as_ref().unwrap();
    assert_eq!(queue.qlen, 0);
    let stats = tc.attr.stats.as_ref().unwrap();
    assert_eq!(stats.bytes, 0);
    assert_eq!(stats.packets, 0);
}

#[test]
fn test_mq() {
    let messages = vec![get_qdiscs()[1].clone()];
    let stats = OpenOptions::new().tc(messages).unwrap();

    let tc = stats.get(0).unwrap();
    // message
    assert_eq!(tc.msg.index, 2);
    assert_eq!(tc.msg.handle, 0);
    assert_eq!(tc.msg.parent, 4294967295);
    // attr
    assert_eq!(tc.attr.kind.as_str(), "mq");
    let basic = tc.attr.stats2.as_ref().unwrap().basic.as_ref().unwrap();
    assert_eq!(basic.bytes, 122851868);
    assert_eq!(basic.packets, 407415);
    let queue = tc.attr.stats2.as_ref().unwrap().queue.as_ref().unwrap();
    assert_eq!(queue.qlen, 0);
    assert_eq!(queue.overlimits, 13);
    let stats = tc.attr.stats.as_ref().unwrap();
    assert_eq!(stats.bytes, 122851868);
    assert_eq!(stats.packets, 407415);
}

#[test]
fn test_fq_codel() {
    let messages = vec![get_qdiscs()[2].clone()];
    let stats = OpenOptions::new().tc(messages).unwrap();

    let tc = stats.get(0).unwrap();
    // message
    assert_eq!(tc.msg.index, 2);
    assert_eq!(tc.msg.handle, 0);
    assert_eq!(tc.msg.parent, 2);
    // attr
    assert_eq!(tc.attr.kind.as_str(), "fq_codel");
    let basic = tc.attr.stats2.as_ref().unwrap().basic.as_ref().unwrap();
    assert_eq!(basic.bytes, 39902796);
    assert_eq!(basic.packets, 165687);
    let queue = tc.attr.stats2.as_ref().unwrap().queue.as_ref().unwrap();
    assert_eq!(queue.qlen, 0);
    assert_eq!(queue.overlimits, 7);
    let stats = tc.attr.stats.as_ref().unwrap();
    assert_eq!(stats.bytes, 39902796);
    assert_eq!(stats.packets, 165687);

    // qdisc
    let fq_codel = tc.attr.qdisc.as_ref().unwrap();
    assert_eq!(
        fq_codel,
        &QDisc::FqCodel(FqCodel {
            target: 4999,
            limit: 10240,
            interval: 99999,
            ecn: 1,
            flows: 1024,
            quantum: 1514,
            ce_threshold: 0,
            drop_batch_size: 64,
            memory_limit: 33554432,
        })
    );
    // xstats
    let xstats = tc.attr.xstats.as_ref().unwrap();
    assert_eq!(
        xstats,
        &XStats::FqCodel(FqCodelXStats {
            maxpacket: 258,
            drop_overlimit: 0,
            ecn_mark: 0,
            new_flow_count: 91,
            new_flows_len: 0,
            old_flows_len: 0,
            ce_mark: 0,
            memory_usage: 0,
            drop_overmemory: 0,
        })
    );
}

#[test]
fn test_htb() {
    let qdiscs = get_qdiscs();
    let classes = get_classes();
    let messages = vec![qdiscs[3].clone(), classes[0].clone()];
    let tc_stats = OpenOptions::new().tc(messages).unwrap();

    let tc = tc_stats.get(0).unwrap();
    // message
    assert_eq!(tc.msg.index, 3);
    assert_eq!(tc.msg.handle, 65536);
    assert_eq!(tc.msg.parent, 4294967295);
    // attr
    assert_eq!(tc.attr.kind.as_str(), "htb");
    let basic = tc.attr.stats2.as_ref().unwrap().basic.as_ref().unwrap();
    assert_eq!(basic.bytes, 0);
    assert_eq!(basic.packets, 0);
    let queue = tc.attr.stats2.as_ref().unwrap().queue.as_ref().unwrap();
    assert_eq!(queue.qlen, 0);
    let stats = tc.attr.stats.as_ref().unwrap();
    assert_eq!(stats.overlimits, 0);

    // qdisc
    let htb = tc.attr.qdisc.as_ref().unwrap();
    assert_eq!(
        htb,
        &QDisc::Htb(HtbGlob {
            version: 196625,
            rate2quantum: 10,
            defcls: 32,
            debug: 0,
            direct_pkts: 0,
        })
    );

    let tc = tc_stats.get(1).unwrap();
    // message
    assert_eq!(tc.msg.index, 3);
    assert_eq!(tc.msg.handle, 65537);
    assert_eq!(tc.msg.parent, 4294967295);

    // class
    let htb = tc.attr.class.as_ref().unwrap();
    assert_eq!(
        htb,
        &Class::Htb(Htb {
            parms: Some(HtbOpt {
                rate: RateSpec {
                    cell_log: 0,
                    linklayer: 1,
                    overhead: 0,
                    cell_align: 0,
                    mpu: 0,
                    rate: 125000,
                },
                ceil: RateSpec {
                    cell_log: 0,
                    linklayer: 1,
                    overhead: 0,
                    cell_align: 0,
                    mpu: 0,
                    rate: 125000,
                },
                buffer: 200000,
                cbuffer: 200000,
                quantum: 12500,
                level: 7,
                prio: 0,
            }),
            init: None,
            ctab: vec![],
            rtab: vec![],
            direct_qlen: None,
            rate64: None,
            ceil64: None,
        })
    );
    // xstats
    let xstats = tc.attr.xstats.as_ref().unwrap();
    assert_eq!(
        xstats,
        &XStats::Htb(HtbXstats {
            lends: 0,
            borrows: 0,
            giants: 0,
            tokens: 200000,
            ctokens: 200000,
        })
    );
}

#[test]
fn test_links() {
    let links = OpenOptions::new().links(get_links()).unwrap();

    assert_eq!(links[0].index, 1);
    assert_eq!(links[0].name, "eth0");
}

#[test]
fn test_unknown_netlink_msg_fail() {
    let messages = vec![NetlinkMessage::new(
        NetlinkHeader::default(),
        NetlinkPayload::InnerMessage(RtnlMessage::DelQueueDiscipline(TcMessage::default())),
    )];
    let stats = OpenOptions::new()
        .fail_on_unknown_netlink_message(true)
        .tc(messages);

    assert!(stats.is_err());
}

#[test]
fn test_unknown_attribute_fail() {
    // hwoffload not implemented
    let tc_message = qdisc("fq_codel");
    let messages = NetlinkMessage::new(
        NetlinkHeader::default(),
        NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(tc_message)),
    );
    let stats = OpenOptions::new()
        .fail_on_unknown_attribute(true)
        .tc(vec![messages]);

    assert!(matches!(
        stats.unwrap_err(),
        Error::Parse(_)
    ));
}

#[test]
fn test_stats_parse_fail() {
    use netlink_packet_route::tc;

    let kind = "fq_codel";
    let mut tc_message = qdisc(kind);
    let mut nlas = nlas(kind);
    // mess up the stats
    nlas[3] = tc::Nla::Stats2(vec![
        tc::nlas::Stats2::StatsBasic(vec![1, 2, 3, 4]),
        tc::nlas::Stats2::StatsQueue(vec![1, 2, 3, 4]),
    ]);
    tc_message.nlas = nlas;
    let messages = NetlinkMessage::new(
        NetlinkHeader::default(),
        NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(tc_message)),
    );
    let tcs = OpenOptions::new().tc(vec![messages]).unwrap();
    let tc = tcs.get(0).unwrap();
    assert!(tc.attr.stats2.is_none());
}

#[test]
fn test_unknown_option_fail() {
    let messages = vec![NetlinkMessage::new(
        NetlinkHeader::default(),
        NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(qdisc("unknown"))),
    )];
    let stats = OpenOptions::new().fail_on_unknown_option(true).tc(messages);

    assert!(stats.is_err());
}
