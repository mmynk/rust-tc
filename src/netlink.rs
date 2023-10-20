use netlink_packet_core::{
    NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_packet_route::{LinkMessage, RtnlMessage, TcHeader, TcMessage};
use netlink_sys::{protocols::NETLINK_ROUTE, Socket, SocketAddr};

use crate::errors::NetlinkError;

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
    fn qdiscs(&self) -> Result<Vec<TcMessage>, NetlinkError>;

    /// Get all classes from Netlink.
    fn classes(&self, index: i32) -> Result<Vec<TcMessage>, NetlinkError>;

    /// Get all links from Netlink.
    fn links(&self) -> Result<Vec<LinkMessage>, NetlinkError>;
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

    fn qdiscs(&self) -> Result<Vec<TcMessage>, NetlinkError> {
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
                    NetlinkPayload::Done(_) => return Ok(tc_messages),
                    _ => {}
                }

                offset += rx_packet.header.length as usize;
                if offset == size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }

        Ok(tc_messages)
    }

    fn classes(&self, index: i32) -> Result<Vec<TcMessage>, NetlinkError> {
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
                    NetlinkPayload::Done(_) => return Ok(tc_messages),
                    _ => {}
                }

                offset += rx_packet.header.length as usize;
                if offset == size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }

        Ok(tc_messages)
    }

    fn links(&self) -> Result<Vec<LinkMessage>, NetlinkError> {
        send_get_link_request(&self.socket)?;

        let mut receive_buffer = vec![0; 4096];
        let mut offset = 0;

        let mut tc_messages = Vec::new();
        while let Ok(size) = self.socket.recv(&mut &mut receive_buffer[..], 0) {
            loop {
                let bytes = &receive_buffer[offset..];
                let rx_packet = <NetlinkMessage<RtnlMessage>>::deserialize(bytes).unwrap();

                let payload = rx_packet.payload;
                match payload {
                    NetlinkPayload::InnerMessage(RtnlMessage::NewLink(message)) => {
                        tc_messages.push(message.clone())
                    }
                    NetlinkPayload::Error(error) => {
                        return Err(NetlinkError::Netlink(error.to_string()))
                    }
                    NetlinkPayload::Done(_) => return Ok(tc_messages),
                    _ => {}
                }

                offset += rx_packet.header.length as usize;
                if offset == size || rx_packet.header.length == 0 {
                    offset = 0;
                    break;
                }
            }
        }

        Ok(tc_messages)
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

    let tc_hdr = TcHeader {
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
