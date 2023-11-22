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
//! use netlink_tc::ParseOptions;
//!
//! // Retrieve netlink messages using `netlink-packet-route`.
//! // See `examples` for more details.
//! let messages: Vec<NetlinkMessage<RtnlMessage>> = vec![]; // init with netlink messages
//!
//! // Get list of tc qdiscs or classes
//! let qdiscs = ParseOptions::new()
//!     .fail_on_unknown_attribute(false)
//!     .fail_on_unknown_option(false)
//!     .tc(messages)
//!     .unwrap();
//! ```
use netlink_packet_core::{NetlinkMessage, NetlinkPayload};
use netlink_packet_route::{tc as netlink_tc, RtnlMessage, TcMessage as NlTcMessage};
use netlink_packet_utils::{nla::Nla, Emitable};

use errors::Error;
use types::{Tc, TcAttr, TcHeader, TcMsg, TcOption, TcStats2};

pub mod errors;
pub mod types;

mod class;
mod constants;
mod qdiscs;
mod tc;

#[cfg(test)]
mod test_data;
#[cfg(test)]
mod tests;

/// Possible message types for `tc` messages.
/// A subset of `rtnl::RtnlMessage` enum.
pub enum RtNetlinkMessage {
    GetQdisc(TcMsg), /* RTM_GETQDISC */
    GetClass(TcMsg), /* RTM_GETCLASS */
}

/// `OpenOptions` provides options for controlling how `netlink-tc` parses netlink messages.
/// By default, unknown attributes and options are ignored.
#[derive(Debug)]
pub struct ParseOptions {
    fail_on_unknown_netlink_message: bool,
    fail_on_unknown_attribute: bool,
    fail_on_unknown_option: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseOptions {
    /// Creates a new set of options with all flags set to false.
    /// By default, the call fails on unknown netlink messages, attributes or options.
    ///
    /// NOTE: Using the default options will lead to the calls failing until the library is complete.
    /// The caller must explicitly set the required options to false until then.
    pub fn new() -> Self {
        Self {
            fail_on_unknown_netlink_message: true,
            fail_on_unknown_attribute: true,
            fail_on_unknown_option: true,
        }
    }

    /// Sets the `fail_on_unknown_netlink_message` flag.
    /// If set to true, `netlink-tc` will return an error if it encounters an unknown netlink message
    /// while parsing `Vec<NetlinkMessage<RtnlMessage>>`.
    pub fn fail_on_unknown_netlink_message(&mut self, fail: bool) -> &mut Self {
        self.fail_on_unknown_netlink_message = fail;
        self
    }

    /// Sets the `fail_on_unknown_tc_attribute` flag.
    /// If set to true, `netlink-tc` will return an error if it encounters an unknown tc attribute.
    pub fn fail_on_unknown_attribute(&mut self, fail: bool) -> &mut Self {
        self.fail_on_unknown_attribute = fail;
        self
    }

    /// Sets the `fail_on_unknown_tc_option` flag.
    /// If set to true, `netlink-tc` will return an error if it encounters an unknown tc option.
    pub fn fail_on_unknown_option(&mut self, fail: bool) -> &mut Self {
        self.fail_on_unknown_option = fail;
        self
    }

    /// Parses `tc` queueing disciplines and classes for the corresponding Netlink messages
    /// with the options specified by `self`.
    ///
    /// # Example
    /// ```no_run
    /// use netlink_tc::ParseOptions;
    ///
    /// let queues = ParseOptions::new()
    ///     .fail_on_unknown_netlink_message(false)
    ///     .fail_on_unknown_attribute(false)
    ///     .fail_on_unknown_option(false)
    ///     .tc(vec![]); // init with netlink messages
    /// ```
    pub fn tc(&self, messages: Vec<NetlinkMessage<RtnlMessage>>) -> Result<Vec<Tc>, Error> {
        tc_stats(messages, self)
    }
}

fn to_tc(tc_message: NlTcMessage, opts: &ParseOptions) -> Result<TcMsg, Error> {
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
                let mut options = Vec::new();
                for opt in tc_opts {
                    match opt {
                        netlink_tc::TcOpt::Other(nla) => {
                            let mut buf = vec![0u8; nla.value_len()];
                            nla.emit_value(buf.as_mut_slice());
                            let option = TcOption {
                                kind: nla.kind(),
                                bytes: buf,
                            };
                            options.push(option);
                        }
                        _ => {
                            if opts.fail_on_unknown_option {
                                return Err(Error::Parse(format!(
                                    "Option {:?} not implemented",
                                    opt
                                )));
                            }
                        }
                    };
                }
                attrs.push(TcAttr::Options(options));
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
                        _ => {
                            if opts.fail_on_unknown_attribute {
                                return Err(Error::Parse(format!(
                                    "Stats2 {:?} not implemented",
                                    stat
                                )));
                            }
                        }
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
            _ => {
                if opts.fail_on_unknown_attribute {
                    return Err(Error::Parse(format!("Attribute {:?} not implemented", nla)));
                }
            }
        }
    }

    Ok(TcMsg { header, attrs })
}

fn parse(
    messages: Vec<NetlinkMessage<RtnlMessage>>,
    opts: &ParseOptions,
) -> Result<Vec<RtNetlinkMessage>, Error> {
    let mut tc_messages = Vec::new();
    for message in messages {
        match message.payload {
            NetlinkPayload::InnerMessage(RtnlMessage::NewQueueDiscipline(message)) => {
                tc_messages.push(RtNetlinkMessage::GetQdisc(to_tc(message.clone(), opts)?))
            }
            NetlinkPayload::InnerMessage(RtnlMessage::NewTrafficClass(message)) => {
                tc_messages.push(RtNetlinkMessage::GetClass(to_tc(message.clone(), opts)?))
            }
            payload => {
                if opts.fail_on_unknown_netlink_message {
                    return Err(Error::Parse(format!(
                        "Unknown netlink message type: {}",
                        payload.message_type()
                    )));
                }
            }
        }
    }
    Ok(tc_messages)
}

/// Parse `tc` queueing disciplines and classes for the corresponding Netlink messages.
fn tc_stats(
    messages: Vec<NetlinkMessage<RtnlMessage>>,
    opts: &ParseOptions,
) -> Result<Vec<Tc>, Error> {
    let messages = parse(messages, opts)?;
    tc::tc_stats(messages, opts)
}
