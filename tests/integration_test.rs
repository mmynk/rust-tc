use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{LinkMessage, RtnlMessage, TcHeader, TcMessage};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};
use netlink_tc::ParseOptions;

fn socket() -> Socket {
    let socket = Socket::new(NETLINK_ROUTE).unwrap();
    socket.connect(&SocketAddr::new(0, 0)).unwrap();
    socket
}

fn receive_netlink_messages(message: RtnlMessage) -> Vec<NetlinkMessage<RtnlMessage>> {
    let socket = socket();
    send_request(&socket, message);

    let mut receive_buffer = vec![0; 4096];
    let mut offset = 0;

    let mut messages = Vec::new();
    while let Ok(size) = socket.recv(&mut &mut receive_buffer[..], 0) {
        loop {
            let bytes = &receive_buffer[offset..];
            let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes).unwrap();
            messages.push(rx_packet.clone());
            let payload = rx_packet.payload;
            if let NetlinkPayload::Error(err) = payload {
                panic!("Netlink error: {:?}", err);
            }
            if let NetlinkPayload::Done(_) = payload {
                return messages;
            }

            offset += rx_packet.header.length as usize;
            if offset == size || rx_packet.header.length == 0 {
                offset = 0;
                break;
            }
        }
    }

    messages
}

fn get_qdiscs() -> Vec<NetlinkMessage<RtnlMessage>> {
    receive_netlink_messages(RtnlMessage::GetQueueDiscipline(TcMessage::default()))
}

fn get_classes(index: u32) -> Vec<NetlinkMessage<RtnlMessage>> {
    let header = TcHeader {
        index: index as i32,
        ..Default::default()
    };
    let mut message = TcMessage::default();
    message.header = header;
    receive_netlink_messages(RtnlMessage::GetTrafficClass(message))
}

fn get_links() -> Vec<NetlinkMessage<RtnlMessage>> {
    receive_netlink_messages(RtnlMessage::GetLink(LinkMessage::default()))
}

fn send_request(socket: &Socket, message: RtnlMessage) {
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_DUMP;

    let mut packet = NetlinkMessage::new(nl_hdr, NetlinkPayload::from(message));
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);

    socket.send(&buf[..], 0).unwrap();
}

#[test]
fn test_qdiscs() {
    let messages = get_qdiscs();
    let tcs = ParseOptions::new()
        .fail_on_unknown_netlink_message(false)
        .fail_on_unknown_attribute(false)
        .fail_on_unknown_option(false)
        .tc(messages)
        .unwrap();
    for tc in tcs {
        let attr = tc.attr;
        assert!(!attr.kind.is_empty());
        assert!(attr.stats.is_some());
        assert!(attr.stats2.is_some());
    }
}

#[test]
fn test_link_classes() {
    let messages = get_links();
    let links = ParseOptions::new()
        .fail_on_unknown_netlink_message(false)
        .fail_on_unknown_attribute(false)
        .fail_on_unknown_option(false)
        .links(messages)
        .unwrap();

    assert!(!links.is_empty());

    for link in links {
        let messages = get_classes(link.index);
        let classes = ParseOptions::new()
            .fail_on_unknown_netlink_message(false)
            .fail_on_unknown_attribute(false)
            .fail_on_unknown_option(false)
            .tc(messages);
        assert!(classes.is_ok());
    }
}
