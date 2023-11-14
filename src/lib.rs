//! # netlink-tc
//!
//! `netlink-tc` provides a pure Rust API for interacting with the [netlink](https://www.kernel.org/doc/html/latest/userspace-api/netlink/intro.html) based Linux Traffic Control ([`tc`](http://man7.org/linux/man-pages/man8/tc.8.html)) subsystem of [`rtnetlink`](http://man7.org/linux/man-pages/man7/rtnetlink.7.html).
//!
//! This library is very much in progress. It only supports a small subset of `classless` and `classful` [qdiscs](https://tldp.org/HOWTO/Traffic-Control-HOWTO/components.html#c-qdisc). Also, the library only supports read at the moment.
//!
//! ## Example
//!
//! ```rust
//! use netlink_packet_core::NetlinkMessage;
//! use netlink_packet_route::RtnlMessage;
//! use netlink_tc as tc;
//!
//! // Retrive netlink messages using `netlink-packet-route`.
//! // See `examples` for more details.
//! let messages: Vec<NetlinkMessage<RtnlMessage>> = vec![]; // init with netlink messages
//!
//! // Get list of tc qdiscs or classes
//! let qdiscs = tc::tc_stats(messages.clone()).unwrap();
//!
//! // Get list of links
//! let links = tc::links(messages.clone()).unwrap();
//! ```

pub use class::*;
pub use errors::*;
pub use qdiscs::*;
pub use types::*;

use netlink_packet_core::{NetlinkMessage, NetlinkPayload};
use netlink_packet_route::{
    link as netlink_link, tc as netlink_tc, LinkMessage as NlLinkMessage, RtnlMessage,
    TcMessage as NlTcMessage,
};
use netlink_packet_utils::{nla::Nla, Emitable};

pub mod errors;
pub mod types;

mod class;
mod constants;
mod link;
mod qdiscs;
mod tc;

#[cfg(test)]
mod test_data;
#[cfg(test)]
mod tests;

/// Possible message types for `tc` messages.
/// A subset of `rtnl::RtnlMessage` enum.
pub enum RtNetlinkMessage {
    GetQdisc(TcMsg),  // RTM_GETQDISC
    GetClass(TcMsg),  // RTM_GETCLASS
    GetLink(LinkMsg), // RTM_GETLINK
}

fn to_tc(tc_message: NlTcMessage) -> TcMsg {
    let NlTcMessage {
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
}

fn to_link(link_message: NlLinkMessage) -> LinkMsg {
    let NlLinkMessage {
        header: link_header,
        nlas,
        ..
    } = link_message;
    let header = LinkHeader {
        index: link_header.index,
    };

    let mut name = String::new();
    for nla in nlas {
        if let netlink_link::nlas::Nla::IfName(if_name) = nla {
            name = if_name;
        }
    }

    LinkMsg {
        header,
        attr: LinkAttr { name },
    }
}

fn parse(
    messages: Vec<NetlinkMessage<RtnlMessage>>,
) -> Result<Vec<RtNetlinkMessage>, NetlinkError> {
    let mut tc_messages = Vec::new();
    for message in messages {
        match message.payload {
            NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(message)) => {
                tc_messages.push(RtNetlinkMessage::GetQdisc(to_tc(message.clone())))
            }
            NetlinkPayload::InnerMessage(RtnlMessage::NewTrafficClass(message)) => {
                tc_messages.push(RtNetlinkMessage::GetClass(to_tc(message.clone())))
            }
            NetlinkPayload::InnerMessage(RtnlMessage::NewLink(message)) => {
                tc_messages.push(RtNetlinkMessage::GetLink(to_link(message.clone())))
            }
            _ => (),
        }
    }
    Ok(tc_messages)
}

/// Parse `tc` queueing disciplines and classes for the corresponding Netlink messages.
pub fn tc_stats(messages: Vec<NetlinkMessage<RtnlMessage>>) -> Result<Vec<Tc>, TcError> {
    let messages = parse(messages).map_err(TcError::Netlink)?;
    tc::tc_stats(messages)
}

/// Parse `link` messages for the corresponding Netlink messages
pub fn links(messages: Vec<NetlinkMessage<RtnlMessage>>) -> Result<Vec<Link>, LinkError> {
    let messages = parse(messages).map_err(LinkError::Netlink)?;
    link::links(messages)
}
