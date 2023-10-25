use std::vec;

use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{link, tc as netlink_tc, LinkMessage, RtnlMessage, TcMessage};
use netlink_packet_utils::{nla::Nla, Emitable};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};

use crate::{errors::NetlinkError, types::*};

/// A trait for a netlink connection.
///
/// This trait allows for mocking the netlink connection in tests.
pub trait NetlinkConnection {
    /// Create a new netlink connection.
    /// Initialize a new netlink socket and connect to the kernel.
    fn new() -> Result<Self, NetlinkError>
    where
        Self: Sized;

    /// Get all qdiscs from Netlink.
    fn qdiscs(&self) -> Result<Vec<TcMsg>, NetlinkError>;

    /// Get all classes from Netlink.
    fn classes(&self, index: i32) -> Result<Vec<TcMsg>, NetlinkError>;

    /// Get all links from Netlink.
    fn links(&self) -> Result<Vec<LinkMsg>, NetlinkError>;
}

/// A struct for communicating with the kernel via netlink.
pub struct Netlink {
    socket: Socket,
}

impl NetlinkConnection for Netlink {
    fn new() -> Result<Self, NetlinkError>
    where
        Self: Sized,
    {
        let socket =
            Socket::new(NETLINK_ROUTE).map_err(|err| NetlinkError::Socket(Box::new(err)))?;
        socket
            .connect(&SocketAddr::new(0, 0))
            .map_err(|err| NetlinkError::Socket(Box::new(err)))?;
        Ok(Self { socket })
    }

    fn qdiscs(&self) -> Result<Vec<TcMsg>, NetlinkError> {
        send_get_qdisc_request(&self.socket)?;

        let mut receive_buffer = vec![0; 4096];
        let mut offset = 0;

        let mut tc_messages = Vec::new();
        while let Ok(size) = self.socket.recv(&mut &mut receive_buffer[..], 0) {
            loop {
                let bytes = &receive_buffer[offset..];
                let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes).unwrap();

                let payload = rx_packet.payload;
                match payload {
                    NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(message)) => {
                        tc_messages.push(message.clone())
                    }
                    NetlinkPayload::Error(error) => {
                        return Err(NetlinkError::Netlink(error.to_string()))
                    }
                    NetlinkPayload::Done(_) => return Ok(to_tc(tc_messages)),
                    _ => {}
                }

                offset += rx_packet.header.length as usize;
                if offset == size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }

        Ok(to_tc(tc_messages))
    }

    fn classes(&self, index: i32) -> Result<Vec<TcMsg>, NetlinkError> {
        send_get_class_request(&self.socket, index)?;

        let mut receive_buffer = vec![0; 4096];
        let mut offset = 0;

        let mut tc_messages = Vec::new();
        while let Ok(size) = self.socket.recv(&mut &mut receive_buffer[..], 0) {
            loop {
                let bytes = &receive_buffer[offset..];
                let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes).unwrap();

                let payload = rx_packet.payload;
                match payload {
                    NetlinkPayload::InnerMessage(RtnlMessage::NewTrafficClass(message)) => {
                        tc_messages.push(message.clone())
                    }
                    NetlinkPayload::Error(error) => {
                        return Err(NetlinkError::Netlink(error.to_string()))
                    }
                    NetlinkPayload::Done(_) => return Ok(to_tc(tc_messages)),
                    _ => {}
                }

                offset += rx_packet.header.length as usize;
                if offset == size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }

        Ok(to_tc(tc_messages))
    }

    fn links(&self) -> Result<Vec<LinkMsg>, NetlinkError> {
        send_get_link_request(&self.socket)?;

        let mut receive_buffer = vec![0; 4096];
        let mut offset = 0;

        let mut link_messages = Vec::new();
        while let Ok(size) = self.socket.recv(&mut &mut receive_buffer[..], 0) {
            loop {
                let bytes = &receive_buffer[offset..];
                let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes).unwrap();

                let payload = rx_packet.payload;
                match payload {
                    NetlinkPayload::InnerMessage(RtnlMessage::NewLink(message)) => {
                        link_messages.push(message.clone())
                    }
                    NetlinkPayload::Error(error) => {
                        return Err(NetlinkError::Netlink(error.to_string()))
                    }
                    NetlinkPayload::Done(_) => return Ok(to_link(link_messages)),
                    _ => {}
                }

                offset += rx_packet.header.length as usize;
                if offset == size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }

        Ok(to_link(link_messages))
    }
}

fn send_get_qdisc_request(socket: &Socket) -> Result<(), NetlinkError> {
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_DUMP;

    let mut packet = NetlinkMessage::new(
        nl_hdr,
        NetlinkPayload::from(RtnlMessage::GetQueueDiscipline(TcMessage::default())),
    );
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);

    match socket.send(&buf[..], 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(NetlinkError::Send(e.to_string())),
    }
}

fn send_get_class_request(socket: &Socket, index: i32) -> Result<(), NetlinkError> {
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_DUMP;

    let tc_hdr = netlink_tc::TcHeader {
        index,
        ..Default::default()
    };
    let mut tc_msg = TcMessage::default();
    tc_msg.header = tc_hdr;
    let mut packet = NetlinkMessage::new(
        nl_hdr,
        NetlinkPayload::from(RtnlMessage::GetTrafficClass(tc_msg)),
    );
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);

    match socket.send(&buf[..], 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(NetlinkError::Send(e.to_string())),
    }
}

fn send_get_link_request(socket: &Socket) -> Result<(), NetlinkError> {
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_DUMP;

    let mut packet = NetlinkMessage::new(
        nl_hdr,
        NetlinkPayload::from(RtnlMessage::GetLink(LinkMessage::default())),
    );
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);

    match socket.send(&buf[..], 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(NetlinkError::Send(e.to_string())),
    }
}

fn to_tc(tc_messages: Vec<TcMessage>) -> Vec<TcMsg> {
    tc_messages
        .into_iter()
        .map(|tc_message| {
            let TcMessage {
                header: tc_header,
                nlas,
                ..
            } = tc_message;
            let header = TcHeader {
                index: tc_header.index,
                handle: tc_header.handle,
                parent: tc_header.parent,
            };
            let mut attrs = Vec::new();

            for nla in nlas {
                match nla {
                    netlink_tc::Nla::Kind(kind) => attrs.push(TcAttr::Kind(kind)),
                    netlink_tc::Nla::Options(tc_opts) => {
                        let mut opts = Vec::new();
                        for opt in tc_opts {
                            match opt {
                                netlink_tc::TcOpt::Ingress => {
                                    let option = TcOption {
                                        kind: 0u16, // TODO: what is Ingress kind?
                                        bytes: vec![],
                                    };
                                    opts.push(option);
                                }
                                netlink_tc::TcOpt::U32(nla) => {
                                    let mut buf = vec![0u8; nla.value_len()];
                                    nla.emit_value(buf.as_mut_slice());
                                    let option = TcOption {
                                        kind: nla.kind(),
                                        bytes: buf,
                                    };
                                    opts.push(option);
                                }
                                netlink_tc::TcOpt::Matchall(nla) => {
                                    let mut buf = vec![0u8; nla.value_len()];
                                    nla.emit_value(buf.as_mut_slice());
                                    let option = TcOption {
                                        kind: nla.kind(),
                                        bytes: buf,
                                    };
                                    opts.push(option);
                                }
                                netlink_tc::TcOpt::Other(nla) => {
                                    let mut buf = vec![0u8; nla.value_len()];
                                    nla.emit_value(buf.as_mut_slice());
                                    let option = TcOption {
                                        kind: nla.kind(),
                                        bytes: buf,
                                    };
                                    opts.push(option);
                                }
                                _ => (),
                            };
                        }
                        attrs.push(TcAttr::Options(opts));
                    }
                    netlink_tc::Nla::Stats(tc_stats) => {
                        let mut buf = vec![0u8; tc_stats.buffer_len()];
                        tc_stats.emit(buf.as_mut_slice());
                        attrs.push(TcAttr::Stats(buf));
                    }
                    netlink_tc::Nla::Stats2(tc_stats) => {
                        let mut stats2 = Vec::new();
                        for stat in tc_stats {
                            match stat {
                                netlink_tc::Stats2::StatsBasic(bytes) => {
                                    stats2.push(TcStats2::StatsBasic(bytes))
                                }
                                netlink_tc::Stats2::StatsQueue(bytes) => {
                                    stats2.push(TcStats2::StatsQueue(bytes))
                                }
                                netlink_tc::Stats2::StatsApp(bytes) => {
                                    stats2.push(TcStats2::StatsApp(bytes))
                                }
                                _ => (),
                            }
                        }
                        attrs.push(TcAttr::Stats2(stats2));
                    }
                    netlink_tc::Nla::XStats(bytes) => attrs.push(TcAttr::Xstats(bytes)),
                    netlink_tc::Nla::Rate(bytes) => attrs.push(TcAttr::Rate(bytes)),
                    netlink_tc::Nla::Fcnt(bytes) => attrs.push(TcAttr::Fcnt(bytes)),
                    netlink_tc::Nla::Stab(bytes) => attrs.push(TcAttr::Stab(bytes)),
                    netlink_tc::Nla::Chain(bytes) => attrs.push(TcAttr::Chain(bytes)),
                    netlink_tc::Nla::HwOffload(byte) => attrs.push(TcAttr::HwOffload(byte)),
                    _ => (),
                }
            }

            TcMsg { header, attrs }
        })
        .collect()
}

fn to_link(link_messages: Vec<LinkMessage>) -> Vec<LinkMsg> {
    link_messages
        .into_iter()
        .map(|link_message| {
            let LinkMessage {
                header: link_header,
                nlas,
                ..
            } = link_message;
            let header = LinkHeader {
                index: link_header.index,
            };

            let mut name = String::new();
            for nla in nlas {
                match nla {
                    link::nlas::Nla::IfName(if_name) => name = if_name,
                    _ => (),
                }
            }

            LinkMsg {
                header,
                attr: LinkAttr { name },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_data;

    #[test]
    fn test_qdiscs_to_tc() {
        let qdiscs = test_data::nl_qdiscs();
        let tcs = to_tc(qdiscs);

        // noqueue
        // TcMessage { header: TcHeader { family: 0, index: 1, handle: 0, parent: 4294967295, info: 2 }, nlas: [Kind("noqueue"), HwOffload(0), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
        fn test_no_queue(tc: &TcMsg) {
            // header
            assert_eq!(tc.header.index, 1);
            assert_eq!(tc.header.handle, 0);
            assert_eq!(tc.header.parent, 4294967295);

            // attr
            let attrs = &tc.attrs;
            for attr in attrs {
                match attr {
                    TcAttr::Kind(kind) => assert_eq!(kind, "noqueue"),
                    TcAttr::Stats(bytes) => assert_eq!(bytes, &vec![0u8; 36]),
                    TcAttr::Stats2(stats) => {
                        for stat in stats {
                            match stat {
                                TcStats2::StatsBasic(bytes) => assert_eq!(bytes, &vec![0u8; 16]),
                                TcStats2::StatsQueue(bytes) => assert_eq!(bytes, &vec![0u8; 20]),
                                _ => (),
                            }
                        }
                    }
                    TcAttr::HwOffload(byte) => assert_eq!(byte, &0),
                    _ => (),
                }
            }
        }

        // mq
        // TcMessage { header: TcHeader { family: 0, index: 2, handle: 0, parent: 4294967295, info: 1 }, nlas: [Kind("mq"), HwOffload(0), Stats2([StatsBasic([28, 146, 82, 7, 0, 0, 0, 0, 119, 55, 6, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 122851868, packets: 407415, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
        fn test_mq(tc: &TcMsg) {
            // header
            assert_eq!(tc.header.index, 2);
            assert_eq!(tc.header.handle, 0);
            assert_eq!(tc.header.parent, 4294967295);

            // attr
            let attrs = &tc.attrs;
            for attr in attrs {
                match attr {
                    TcAttr::Kind(kind) => assert_eq!(kind, "mq"),
                    TcAttr::Stats(bytes) => {
                        assert_eq!(
                            bytes,
                            &vec![
                                28, 146, 82, 7, 0, 0, 0, 0, 119, 55, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            ]
                        );
                    }
                    TcAttr::Stats2(stats) => {
                        for stat in stats {
                            match stat {
                                TcStats2::StatsBasic(bytes) => assert_eq!(
                                    bytes,
                                    &vec![28, 146, 82, 7, 0, 0, 0, 0, 119, 55, 6, 0, 0, 0, 0, 0,]
                                ),
                                TcStats2::StatsQueue(bytes) => assert_eq!(
                                    bytes,
                                    &vec![
                                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0,
                                        0,
                                    ]
                                ),
                                _ => (),
                            }
                        }
                    }
                    TcAttr::HwOffload(byte) => assert_eq!(byte, &0),
                    _ => (),
                }
            }
        }

        // fq_codel
        // TcMessage { header: TcHeader { family: 0, index: 2, handle: 0, parent: 2, info: 1 }, nlas: [Kind("fq_codel"), Options([Other(DefaultNla { kind: 1, value: [135, 19, 0, 0] }), Other(DefaultNla { kind: 2, value: [0, 40, 0, 0] }), Other(DefaultNla { kind: 3, value: [159, 134, 1, 0] }), Other(DefaultNla { kind: 4, value: [1, 0, 0, 0] }), Other(DefaultNla { kind: 6, value: [234, 5, 0, 0] }), Other(DefaultNla { kind: 8, value: [64, 0, 0, 0] }), Other(DefaultNla { kind: 9, value: [0, 0, 0, 2] }), Other(DefaultNla { kind: 5, value: [0, 4, 0, 0] })]), HwOffload(0), Stats2([StatsApp([0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsBasic([76, 222, 96, 2, 0, 0, 0, 0, 55, 135, 2, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 39902796, packets: 165687, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 }), XStats([0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])] }
        fn test_fq_codel(tc: &TcMsg) {
            // header
            assert_eq!(tc.header.index, 2);
            assert_eq!(tc.header.handle, 0);
            assert_eq!(tc.header.parent, 2);

            // attr
            let attrs = &tc.attrs;
            for attr in attrs {
                match attr {
                    TcAttr::Kind(kind) => assert_eq!(kind, "fq_codel"),
                    TcAttr::Options(opts) => {
                        for opt in opts {
                            match opt {
                                TcOption { kind, bytes } => match kind {
                                    1 => assert_eq!(bytes, &vec![135, 19, 0, 0]),
                                    2 => assert_eq!(bytes, &vec![0, 40, 0, 0]),
                                    3 => assert_eq!(bytes, &vec![159, 134, 1, 0]),
                                    4 => assert_eq!(bytes, &vec![1, 0, 0, 0]),
                                    5 => assert_eq!(bytes, &vec![0, 4, 0, 0]),
                                    6 => assert_eq!(bytes, &vec![234, 5, 0, 0]),
                                    8 => assert_eq!(bytes, &vec![64, 0, 0, 0]),
                                    9 => assert_eq!(bytes, &vec![0, 0, 0, 2]),
                                    _ => (),
                                },
                            }
                        }
                    }
                    TcAttr::Stats(bytes) => assert_eq!(
                        bytes,
                        &vec![
                            76, 222, 96, 2, 0, 0, 0, 0, 55, 135, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        ]
                    ),
                    TcAttr::Stats2(stats) => {
                        for stat in stats {
                            match stat {
                                TcStats2::StatsBasic(bytes) => assert_eq!(
                                    bytes,
                                    &vec![76, 222, 96, 2, 0, 0, 0, 0, 55, 135, 2, 0, 0, 0, 0, 0,]
                                ),
                                TcStats2::StatsQueue(bytes) => assert_eq!(
                                    bytes,
                                    &vec![
                                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0,
                                    ]
                                ),
                                TcStats2::StatsApp(bytes) => assert_eq!(
                                    bytes,
                                    &vec![
                                        0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0,
                                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                        0,
                                    ]
                                ),
                            }
                        }
                    }
                    TcAttr::Xstats(bytes) => assert_eq!(
                        bytes,
                        &vec![
                            0, 0, 0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 91, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        ]
                    ),
                    TcAttr::HwOffload(byte) => assert_eq!(byte, &0),
                    _ => (),
                }
            }
        }

        // htb
        // TcMessage { header: TcHeader { family: 0, index: 3, handle: 65536, parent: 4294967295, info: 2 }, nlas: [Kind("htb"), Options([Other(DefaultNla { kind: 2, value: [17, 0, 3, 0, 10, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }), Other(DefaultNla { kind: 5, value: [232, 3, 0, 0] })]), HwOffload(0), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 })] }
        fn test_htb(tc: &TcMsg) {
            // header
            assert_eq!(tc.header.index, 3);
            assert_eq!(tc.header.handle, 65536);
            assert_eq!(tc.header.parent, 4294967295);

            // attr
            let attrs = &tc.attrs;
            for attr in attrs {
                match attr {
                    TcAttr::Kind(kind) => assert_eq!(kind, "htb"),
                    TcAttr::Options(opts) => {
                        for opt in opts {
                            match opt {
                                TcOption { kind, bytes } => match kind {
                                    2 => assert_eq!(
                                        bytes,
                                        &vec![
                                            17, 0, 3, 0, 10, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0,
                                            0, 0, 0,
                                        ]
                                    ),
                                    5 => assert_eq!(bytes, &vec![232, 3, 0, 0]),
                                    _ => (),
                                },
                            }
                        }
                    }
                    TcAttr::Stats(bytes) => assert_eq!(bytes, &vec![0u8; 36]),
                    TcAttr::Stats2(stats2) => {
                        for stat in stats2 {
                            match stat {
                                TcStats2::StatsBasic(bytes) => assert_eq!(bytes, &vec![0u8; 16]),
                                TcStats2::StatsQueue(bytes) => assert_eq!(bytes, &vec![0u8; 20]),
                                _ => (),
                            }
                        }
                    }
                    TcAttr::HwOffload(byte) => assert_eq!(byte, &0),
                    _ => (),
                }
            }
        }

        test_no_queue(tcs.get(0).unwrap());
        test_mq(tcs.get(1).unwrap());
        test_fq_codel(tcs.get(2).unwrap());
        test_htb(tcs.get(3).unwrap());
    }

    #[test]
    fn test_classes_to_tc() {
        let classes = test_data::nl_classes();
        let tcs = to_tc(classes);

        // htb
        // TcMessage { header: TcHeader { family: 0, index: 3, handle: 65537, parent: 4294967295, info: 0 }, nlas: [Kind("htb"), Options([Other(DefaultNla { kind: 1, value: [0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 64, 13, 3, 0, 64, 13, 3, 0, 212, 48, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0] })]), Stats2([StatsBasic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsQueue([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), StatsApp([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13, 3, 0])]), Stats(Stats { bytes: 0, packets: 0, drops: 0, overlimits: 0, bps: 0, pps: 0, qlen: 0, backlog: 0 }), XStats([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13, 3, 0])] }
        fn test_htb(tc: &TcMsg) {
            // header
            assert_eq!(tc.header.index, 3);
            assert_eq!(tc.header.handle, 65537);
            assert_eq!(tc.header.parent, 4294967295);

            // attr
            let attrs = &tc.attrs;
            for attr in attrs {
                match attr {
                    TcAttr::Kind(kind) => assert_eq!(kind, "htb"),
                    TcAttr::Options(opts) => {
                        for opt in opts {
                            match opt {
                                TcOption { kind, bytes } => match kind {
                                    1 => assert_eq!(
                                        bytes,
                                        &vec![
                                            0, 1, 0, 0, 0, 0, 0, 0, 72, 232, 1, 0, 0, 1, 0, 0, 0,
                                            0, 0, 0, 72, 232, 1, 0, 64, 13, 3, 0, 64, 13, 3, 0,
                                            212, 48, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
                                        ]
                                    ),
                                    _ => (),
                                },
                            }
                        }
                    }
                    TcAttr::Stats(bytes) => assert_eq!(bytes, &vec![0u8; 36]),
                    TcAttr::Stats2(stats2) => {
                        for stat in stats2 {
                            match stat {
                                TcStats2::StatsBasic(bytes) => assert_eq!(bytes, &vec![0u8; 16]),
                                TcStats2::StatsQueue(bytes) => assert_eq!(bytes, &vec![0u8; 20]),
                                TcStats2::StatsApp(bytes) => assert_eq!(
                                    bytes,
                                    &vec![
                                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13,
                                        3, 0,
                                    ]
                                ),
                            }
                        }
                    }
                    TcAttr::Xstats(bytes) => assert_eq!(
                        bytes,
                        &vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 13, 3, 0, 64, 13, 3, 0,]
                    ),
                    _ => (),
                }
            }
        }

        test_htb(tcs.get(0).unwrap());
    }

    #[test]
    fn test_to_link() {
        let links = test_data::nl_links();
        let link_msgs = to_link(links);

        let link = link_msgs.get(0).unwrap();
        assert_eq!(link.header.index, 1);
        assert_eq!(link.attr.name, "eth0");
    }
}
