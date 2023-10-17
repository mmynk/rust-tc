mod constants;
mod errors;
mod netlink;
mod qdiscs;
mod tc;
mod types;

pub use netlink::*;
pub use qdiscs::*;
pub use types::*;

pub fn tc_stats<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, errors::TcError> {
    tc::tc_stats::<T>()
}

#[cfg(test)]
mod tests {
    use crate::errors::TcError;
    use netlink_packet_route::{tc, TcHeader, TcMessage};
    use netlink_packet_utils::{nla, Parseable};

    use crate::netlink::NetlinkConnection;

    use super::*;

    struct MockNetlink {}

    impl NetlinkConnection for MockNetlink {
        fn new() -> Result<Self, TcError>
        where
            Self: Sized,
        {
            Ok(MockNetlink {})
        }

        fn qdiscs(&self) -> Result<Vec<TcMessage>, TcError> {
            let mut messages = Vec::with_capacity(3);

            // Create and push the first TcMessage
            // TcMessage { header: TcHeader { family: 0, index: 1, handle: 0, parent: 4294967295, info: 2 }, nlas: [Kind("noqueue"), HwOffload(0), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
            let header = TcHeader {
                family: 0,
                index: 1,
                handle: 0,
                parent: 4294967295,
                info: 2,
            };
            let nlas = vec![
                tc::nlas::Nla::Kind("noqueue".to_string()),
                tc::nlas::Nla::HwOffload(0),
                tc::nlas::Nla::Stats2(vec![
                    tc::nlas::Stats2::StatsBasic(vec![
                        0, 0, 0, 0, 0, 0, 0, 0, // bytes
                        0, 0, 0, 0, // packets
                        0, 0, 0, 0, // padding
                    ]),
                    tc::nlas::Stats2::StatsQueue(vec![
                        0, 0, 0, 0, // qlen
                        0, 0, 0, 0, // backlog
                        0, 0, 0, 0, // drops
                        0, 0, 0, 0, // requeues
                        0, 0, 0, 0, // overlimits
                    ]),
                ]),
                tc::nlas::Nla::Stats({
                    let buf = [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // bytes
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // packets
                        0, 0, 0, 0, // drops
                        0, 0, 0, 0, // overlimits
                        0, 0, 0, 0, // bps
                        0, 0, 0, 0, // pps
                        0, 0, 0, 0, // qlen
                        0, 0, 0, 0, // backlog
                    ];
                    let stats_buf = tc::nlas::StatsBuffer::new(&buf);
                    tc::nlas::Stats::parse(&stats_buf).unwrap()
                }),
            ];
            messages.push(TcMessage::from_parts(header, nlas));

            // Create and push the second TcMessage
            // TcMessage { header: TcHeader { family: 0, index: 2, handle: 0, parent: 4294967295, info: 1 }, nlas: [Kind("mq"), HwOffload(0), Stats2([StatsBasic([28, 146, 82, 7, 0, 0, 0, 0, 119, 55, 6, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 122851868, packets: 407415, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
            let header = TcHeader {
                family: 0,
                index: 2,
                handle: 0,
                parent: 4294967295,
                info: 1,
            };
            let nlas = vec![
                tc::nlas::Nla::Kind("mq".to_string()),
                tc::nlas::Nla::HwOffload(0),
                tc::nlas::Nla::Stats2(vec![
                    tc::nlas::Stats2::StatsBasic(vec![
                        28, 146, 82, 7, 0, 0, 0, 0, // bytes
                        119, 55, 6, 0, // packets
                        0, 0, 0, 0, // padding
                    ]),
                    tc::nlas::Stats2::StatsQueue(vec![
                        0, 0, 0, 0, // qlen
                        0, 0, 0, 0, // backlog
                        0, 0, 0, 0, // drops
                        0, 0, 0, 0, // requeues
                        13, 0, 0, 0, // overlimits
                    ]),
                ]),
                // Stats(Stats { bytes: 122851868, packets: 407415, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })
                tc::nlas::Nla::Stats({
                    let buf = [
                        28, 146, 82, 7, 0, 0, 0, 0, // bytes
                        119, 55, 6, 0, // packets
                        0, 0, 0, 0, // drops
                        0, 0, 0, 0, // overlimits
                        0, 0, 0, 0, // bps
                        0, 0, 0, 0, // pps
                        0, 0, 0, 0, // qlen
                        0, 0, 0, 0, // backlog
                    ];
                    let stats_buf = tc::nlas::StatsBuffer::new(&buf);
                    tc::nlas::Stats::parse(&stats_buf).unwrap()
                }),
            ];
            messages.push(TcMessage::from_parts(header, nlas));

            // Create and push the third TcMessage
            // TcMessage { header: TcHeader { family: 0, index: 2, handle: 0, parent: 2, info: 1 }, nlas: [Kind("fq_codel"), Options([Other(DefaultNla { kind: 1, value: [135, 19, 0, 0] }), Other(DefaultNla { kind: 2, value: [0, 40, 0, 0] }), Other(DefaultNla { kind: 3, value: [159, 134, 1, 0] }), Other(DefaultNla { kind: 4, value: [1, 0, 0, 0] }), Other(DefaultNla { kind: 6, value: [234, 5, 0, 0] }), Other(DefaultNla { kind: 8, value: [64, 0, 0, 0] }), Other(DefaultNla { kind: 9, value: [0, 0, 0, 2] }), Other(DefaultNla { kind: 5, value: [0, 4, 0, 0] })]), HwOffload(0), Stats2([StatsApp([0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsBasic([76, 222, 96, 2, 0, 0, 0, 0, 55, 135, 2, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 39902796, packets: 165687, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 }), XStats([0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])] }
            let header = TcHeader {
                family: 0,
                index: 2,
                handle: 0,
                parent: 2,
                info: 1,
            };
            let nlas = vec![
                tc::nlas::Nla::Kind("fq_codel".to_string()),
                tc::nlas::Nla::Options({
                    let mut opts = Vec::with_capacity(8);
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        1,
                        vec![135, 19, 0, 0],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        2,
                        vec![0, 40, 0, 0],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        3,
                        vec![159, 134, 1, 0],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        4,
                        vec![1, 0, 0, 0],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        6,
                        vec![234, 5, 0, 0],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        8,
                        vec![64, 0, 0, 0],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        9,
                        vec![0, 0, 0, 2],
                    )));
                    opts.push(tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                        5,
                        vec![0, 4, 0, 0],
                    )));
                    opts
                }),
                tc::nlas::Nla::HwOffload(0),
                tc::nlas::Nla::Stats2(vec![
                    tc::nlas::Stats2::StatsApp(vec![
                        0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    ]),
                    tc::nlas::Stats2::StatsBasic(vec![
                        76, 222, 96, 2, 0, 0, 0, 0, // bytes
                        55, 135, 2, 0, // packets
                        0, 0, 0, 0, // padding
                    ]),
                    tc::nlas::Stats2::StatsQueue(vec![
                        0, 0, 0, 0, // qlen
                        0, 0, 0, 0, // backlog
                        0, 0, 0, 0, // drops
                        0, 0, 0, 0, // requeues
                        7, 0, 0, 0, // overlimits
                    ]),
                ]),
                tc::nlas::Nla::Stats({
                    let buf = [
                        76, 222, 96, 2, 0, 0, 0, 0, // bytes
                        55, 135, 2, 0, // packets
                        0, 0, 0, 0, // drops
                        0, 0, 0, 0, // overlimits
                        0, 0, 0, 0, // bps
                        0, 0, 0, 0, // pps
                        0, 0, 0, 0, // qlen
                        0, 0, 0, 0, // backlog
                    ];
                    let stats_buf = tc::nlas::StatsBuffer::new(&buf);
                    tc::nlas::Stats::parse(&stats_buf).unwrap()
                }),
                tc::nlas::Nla::XStats(vec![
                    0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ]),
            ];
            messages.push(TcMessage::from_parts(header, nlas));

            Ok(messages)
        }
    }

    #[test]
    fn test_no_queue() {
        let stats = tc_stats::<MockNetlink>().unwrap();

        let tc = stats.get(0).unwrap();
        // message
        assert_eq!(tc.msg.index, 1);
        assert_eq!(tc.msg.handle, 0);
        assert_eq!(tc.msg.parent, 4294967295);
        // attr
        assert_eq!(tc.attr.kind.as_ref().unwrap(), "noqueue");
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
        let stats = tc_stats::<MockNetlink>().unwrap();

        let tc = stats.get(1).unwrap();
        // message
        assert_eq!(tc.msg.index, 2);
        assert_eq!(tc.msg.handle, 0);
        assert_eq!(tc.msg.parent, 4294967295);
        // attr
        assert_eq!(tc.attr.kind.as_ref().unwrap(), "mq");
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
        let stats = tc_stats::<MockNetlink>().unwrap();

        let tc = stats.get(2).unwrap();
        // message
        assert_eq!(tc.msg.index, 2);
        assert_eq!(tc.msg.handle, 0);
        assert_eq!(tc.msg.parent, 2);
        // attr
        assert_eq!(tc.attr.kind.as_ref().unwrap(), "fq_codel");
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
}
