// use netlink_tc as tc;

// #[test]
// fn test_get_qdiscs() {
//     let result = tc::qdiscs();
//     assert!(result.is_ok());
//     let tcs = result.unwrap();
//     for tc in tcs {
//         let attr = tc.attr;
//         assert!(!attr.kind.is_empty());
//         assert!(attr.stats.is_some());
//         assert!(attr.stats2.is_some());
//     }
// }

// #[test]
// fn test_get_classes() {
//     let result = tc::classes();
//     assert!(result.is_ok());
// }

// #[test]
// fn test_get_links() {
//     let result = tc::link::links();
//     assert!(result.is_ok());
// }

use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{LinkMessage, RtnlMessage, TcHeader, TcMessage};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};
use netlink_tc::{errors::NetlinkError, links, tc_stats};

fn socket() -> Result<Socket, NetlinkError> {
    let socket = Socket::new(NETLINK_ROUTE).map_err(|err| NetlinkError::Socket(Box::new(err)))?;
    socket
        .connect(&SocketAddr::new(0, 0))
        .map_err(|err| NetlinkError::Socket(Box::new(err)))?;
    Ok(socket)
}

fn receive_netlink_messages(
    message: RtnlMessage,
) -> Result<Vec<NetlinkMessage<RtnlMessage>>, NetlinkError> {
    let socket = socket()?;
    send_request(&socket, message)?;

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
                return Err(NetlinkError::Netlink(err.to_string()));
            }
            if let NetlinkPayload::Done(_) = payload {
                return Ok(messages);
            }

            offset += rx_packet.header.length as usize;
            if offset == size || rx_packet.header.length == 0 {
                offset = 0;
                break;
            }
        }
    }

    Ok(messages)
}

fn get_qdiscs() -> Result<Vec<NetlinkMessage<RtnlMessage>>, NetlinkError> {
    receive_netlink_messages(RtnlMessage::GetQueueDiscipline(TcMessage::default()))
}

fn get_classes(index: u32) -> Result<Vec<NetlinkMessage<RtnlMessage>>, NetlinkError> {
    let header = TcHeader {
        index: index as i32,
        ..Default::default()
    };
    let mut message = TcMessage::default();
    message.header = header;
    receive_netlink_messages(RtnlMessage::GetTrafficClass(message))
}

fn get_links() -> Result<Vec<NetlinkMessage<RtnlMessage>>, NetlinkError> {
    receive_netlink_messages(RtnlMessage::GetLink(LinkMessage::default()))
}

fn send_request(socket: &Socket, message: RtnlMessage) -> Result<(), NetlinkError> {
    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_DUMP;

    let mut packet = NetlinkMessage::new(nl_hdr, NetlinkPayload::from(message));
    packet.finalize();

    let mut buf = vec![0; packet.header.length as usize];
    packet.serialize(&mut buf[..]);

    match socket.send(&buf[..], 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(NetlinkError::Send(e.to_string())),
    }
}

#[test]
fn test_qdiscs() {
    let messages = get_qdiscs().unwrap();
    let qdiscs = tc_stats(messages);
    assert!(qdiscs.is_ok());
    let tcs = qdiscs.unwrap();
    for tc in tcs {
        let attr = tc.attr;
        assert!(!attr.kind.is_empty());
        assert!(attr.stats.is_some());
        assert!(attr.stats2.is_some());
    }
}

#[test]
fn test_link_classes() {
    let messages = get_links().unwrap();
    let links = links(messages).unwrap();

    assert!(!links.is_empty());

    for link in links {
        let messages = get_classes(link.index).unwrap();
        let classes = tc_stats(messages);
        assert!(classes.is_ok());
    }
}
