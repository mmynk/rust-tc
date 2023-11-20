use netlink_packet_core::{NetlinkHeader, NetlinkMessage, NetlinkPayload};
use netlink_packet_route::{nlas, tc, LinkMessage, RtnlMessage, TcHeader as NlTcHeader, TcMessage};
use netlink_packet_utils::{nla, Parseable};

pub fn nl_qdiscs() -> Vec<TcMessage> {
    let mut messages = Vec::with_capacity(3);

    // noqueue
    // TcMessage { header: TcHeader { family: 0, index: 1, handle: 0, parent: 4294967295, info: 2 }, nlas: [Kind("noqueue"), HwOffload(0), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
    let header = NlTcHeader {
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

    // mq
    // TcMessage { header: TcHeader { family: 0, index: 2, handle: 0, parent: 4294967295, info: 1 }, nlas: [Kind("mq"), HwOffload(0), Stats2([StatsBasic([28, 146, 82, 7, 0, 0, 0, 0, 119, 55, 6, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 122851868, packets: 407415, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
    let header = NlTcHeader {
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

    // fq_codel
    // TcMessage { header: TcHeader { family: 0, index: 2, handle: 0, parent: 2, info: 1 }, nlas: [Kind("fq_codel"), Options([Other(DefaultNla { kind: 1, value: [135, 19, 0, 0] }), Other(DefaultNla { kind: 2, value: [0, 40, 0, 0] }), Other(DefaultNla { kind: 3, value: [159, 134, 1, 0] }), Other(DefaultNla { kind: 4, value: [1, 0, 0, 0] }), Other(DefaultNla { kind: 6, value: [234, 5, 0, 0] }), Other(DefaultNla { kind: 8, value: [64, 0, 0, 0] }), Other(DefaultNla { kind: 9, value: [0, 0, 0, 2] }), Other(DefaultNla { kind: 5, value: [0, 4, 0, 0] })]), HwOffload(0), Stats2([StatsApp([0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsBasic([76, 222, 96, 2, 0, 0, 0, 0, 55, 135, 2, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 39902796, packets: 165687, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 }), XStats([0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])] }
    let header = NlTcHeader {
        family: 0,
        index: 2,
        handle: 0,
        parent: 2,
        info: 1,
    };
    let nlas = vec![
        tc::nlas::Nla::Kind("fq_codel".to_string()),
        tc::nlas::Nla::Options(vec![
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(1, vec![135, 19, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(2, vec![0, 40, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(3, vec![159, 134, 1, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(4, vec![1, 0, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(6, vec![234, 5, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(8, vec![64, 0, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(9, vec![0, 0, 0, 2])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(5, vec![0, 4, 0, 0])),
        ]),
        tc::nlas::Nla::HwOffload(0),
        tc::nlas::Nla::Stats2(vec![
            tc::nlas::Stats2::StatsApp(vec![
                0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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
            0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]),
    ];
    messages.push(TcMessage::from_parts(header, nlas));

    // htb
    // TcMessage { header: TcHeader { family: 0, index: 3, handle: 65536, parent: 4294967295, info: 2 }, nlas: [Kind("htb"), Options([Other(DefaultNla { kind: 2, value: [17, 0, 3, 0, 10, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }), Other(DefaultNla { kind: 5, value: [232, 3, 0, 0] })]), HwOffload(0), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
    let header = NlTcHeader {
        family: 0,
        index: 3,
        handle: 65536,
        parent: 4294967295,
        info: 2,
    };
    let nlas = vec![
        tc::nlas::Nla::Kind("htb".to_string()),
        tc::nlas::Nla::Options(vec![
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(
                2,
                vec![
                    17, 0, 3, 0, // rate
                    10, 0, 0, 0, // ceil
                    32, 0, 0, 0, // buffer
                    0, 0, 0, 0, // cbuffer
                    0, 0, 0, 0, // quantum
                ],
            )),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(5, vec![232, 3, 0, 0])),
        ]),
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
                0, 0, 0, 0, 0, 0, 0, 0, // bytes
                0, 0, 0, 0, // packets
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

    messages
}

pub fn nl_classes() -> Vec<TcMessage> {
    let mut messages = Vec::with_capacity(1);

    // htb
    // TcMessage { header: TcHeader { family: 0, index: 3, handle: 65537, parent: 4294967295, info: 0 }, nlas: [Kind("htb"), Options([Other(DefaultNla { kind: 1, value: [0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 64, 13, 3, 0, 64, 13, 3, 0, 212, 48, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0] })]), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsApp([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13, 3, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 }), XStats([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13, 3, 0])] }
    let header = NlTcHeader {
        family: 0,
        index: 3,
        handle: 65537,
        parent: 4294967295,
        info: 0,
    };
    let nlas = vec![
        tc::nlas::Nla::Kind("htb".to_string()),
        tc::nlas::Nla::Options(vec![tc::nlas::TcOpt::Other(nla::DefaultNla::new(
            1,
            vec![
                0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 64,
                13, 3, 0, 64, 13, 3, 0, 212, 48, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
            ],
        ))]),
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
            tc::nlas::Stats2::StatsApp(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13, 3, 0,
            ]),
        ]),
        tc::nlas::Nla::Stats({
            let buf = [
                0, 0, 0, 0, 0, 0, 0, 0, // bytes
                0, 0, 0, 0, // packets
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
            0, 0, 0, 0, // lends
            0, 0, 0, 0, // borrows
            0, 0, 0, 0, // giants
            64, 13, 3, 0, // tokens
            64, 13, 3, 0, // ctokens
        ]),
    ];
    messages.push(TcMessage::from_parts(header, nlas));

    messages
}

pub fn nl_links() -> Vec<LinkMessage> {
    let mut msg = LinkMessage::default();
    msg.header.index = 1;
    msg.nlas = vec![nlas::link::Nla::IfName("eth0".to_string())];
    vec![msg]
}

pub fn get_qdiscs() -> Vec<NetlinkMessage<RtnlMessage>> {
    nl_qdiscs()
        .into_iter()
        .map(|qdisc| {
            NetlinkMessage::new(
                NetlinkHeader::default(),
                NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(qdisc)),
            )
        })
        .collect()
}

pub fn get_classes() -> Vec<NetlinkMessage<RtnlMessage>> {
    nl_classes()
        .into_iter()
        .map(|class| {
            NetlinkMessage::new(
                NetlinkHeader::default(),
                NetlinkPayload::InnerMessage(RtnlMessage::NewTrafficClass(class)),
            )
        })
        .collect()
}

pub fn get_links() -> Vec<NetlinkMessage<RtnlMessage>> {
    nl_links()
        .into_iter()
        .map(|link| {
            NetlinkMessage::new(
                NetlinkHeader::default(),
                NetlinkPayload::InnerMessage(RtnlMessage::NewLink(link)),
            )
        })
        .collect()
}

//noinspection DuplicatedCode
pub fn unknown_qdisc() -> NetlinkMessage<RtnlMessage> {
    let header = NlTcHeader {
        family: 0,
        index: 2,
        handle: 0,
        parent: 1,
        info: 1,
    };
    let nlas = vec![
        tc::nlas::Nla::Kind("unknown".to_string()),
        tc::nlas::Nla::Options(vec![
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(1, vec![135, 19, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(2, vec![0, 40, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(3, vec![159, 134, 1, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(4, vec![1, 0, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(6, vec![234, 5, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(8, vec![64, 0, 0, 0])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(9, vec![0, 0, 0, 2])),
            tc::nlas::TcOpt::Other(nla::DefaultNla::new(5, vec![0, 4, 0, 0])),
        ]),
        tc::nlas::Nla::HwOffload(0),
        tc::nlas::Nla::Stats2(vec![
            tc::nlas::Stats2::StatsApp(vec![
                0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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
            0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]),
    ];

    NetlinkMessage::new(
        NetlinkHeader::default(),
        NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(
            TcMessage::from_parts(header, nlas),
        )),
    )
}
