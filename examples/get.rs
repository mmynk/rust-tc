use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{LinkMessage, RtnlMessage, TcHeader, TcMessage};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};
use netlink_tc::OpenOptions;

fn socket() -> Socket {
    let socket = Socket::new(NETLINK_ROUTE).unwrap();
    socket
        .connect(&SocketAddr::new(0, 0))
        .unwrap();
    socket
}

fn receive_netlink_messages(
    message: RtnlMessage,
) -> Vec<NetlinkMessage<RtnlMessage>> {
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

fn get_classes(index: i32) -> Vec<NetlinkMessage<RtnlMessage>> {
    let header = TcHeader {
        index,
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

fn main() {
    let messages = get_qdiscs();
    let qdiscs = OpenOptions::new()
        .tc(messages)
        .unwrap();
    println!("length: {}, qdiscs: {:#?}", qdiscs.len(), qdiscs);

    let messages = get_links();
    let links = OpenOptions::new()
        .links(messages)
        .unwrap();
    println!("length: {}, links: {:#?}", links.len(), links);

    let mut messages = Vec::new();
    for link in links {
        let classes = get_classes(link.index as i32);
        messages.extend(classes);
    }
    let classes = OpenOptions::new()
        .tc(messages)
        .unwrap();
    println!("length: {}, classes: {:#?}", classes.len(), classes);
}
