use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{RtnlMessage, TcHeader, TcMessage};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};
use netlink_tc::ParseOptions;
use nix::ifaddrs::getifaddrs;
use nix::net::if_::if_nametoindex;
use std::collections::BTreeSet;
use std::ffi::OsStr;

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

fn get_classes(index: i32) -> Vec<NetlinkMessage<RtnlMessage>> {
    let header = TcHeader {
        index,
        ..Default::default()
    };
    let mut message = TcMessage::default();
    message.header = header;
    receive_netlink_messages(RtnlMessage::GetTrafficClass(message))
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

fn get_links() -> BTreeSet<i32> {
    if let Ok(addrs) = getifaddrs() {
        addrs
            .filter(|ifaddr| ifaddr.flags.contains(nix::net::if_::InterfaceFlags::IFF_UP))
            .filter_map(|ifaddr| {
                if let Ok(index) = if_nametoindex::<OsStr>(ifaddr.interface_name.as_ref()) {
                    Some(index as i32)
                } else {
                    None
                }
            })
            .collect()
    } else {
        BTreeSet::new()
    }
}

fn main() {
    let messages = get_qdiscs();
    let qdiscs = ParseOptions::new()
        .fail_on_unknown_netlink_message(false)
        .fail_on_unknown_attribute(false)
        .fail_on_unknown_option(false)
        .tc(messages)
        .unwrap();
    println!("length: {}, qdiscs: {:#?}", qdiscs.len(), qdiscs);

    let links = get_links();
    println!("length: {}, links: {:#?}", links.len(), links);

    let mut messages = Vec::new();
    for link in links {
        let classes = get_classes(link);
        messages.extend(classes);
    }
    let classes = ParseOptions::new()
        .fail_on_unknown_netlink_message(false)
        .fail_on_unknown_attribute(false)
        .fail_on_unknown_option(false)
        .tc(messages)
        .unwrap();
    println!("length: {}, classes: {:#?}", classes.len(), classes);
}
