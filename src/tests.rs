use crate::{errors::NetlinkError, netlink::NetlinkConnection};

use super::*;

struct MockNetlink {}

impl NetlinkConnection for MockNetlink {
    fn new() -> Result<Self, NetlinkError>
    where
        Self: Sized,
    {
        Ok(MockNetlink {})
    }

    fn qdiscs(&self) -> Result<Vec<TcMsg>, NetlinkError> {
        Ok(test_data::qdiscs())
    }

    fn classes(&self, _: i32) -> Result<Vec<TcMsg>, NetlinkError> {
        Ok(test_data::classes())
    }

    fn links(&self) -> Result<Vec<LinkMsg>, NetlinkError> {
        Ok(test_data::links())
    }
}

#[test]
fn test_no_queue() {
    let stats = nl_qdiscs::<MockNetlink>().unwrap();

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
    let stats = nl_qdiscs::<MockNetlink>().unwrap();

    let tc = stats.get(1).unwrap();
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
    let stats = nl_qdiscs::<MockNetlink>().unwrap();

    let tc = stats.get(2).unwrap();
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
    let tc_stats = nl_tc_stats::<MockNetlink>().unwrap();

    let tc = tc_stats.get(3).unwrap();
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

    let tc = tc_stats.get(4).unwrap();
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
