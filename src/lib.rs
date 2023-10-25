//! # netlink-tc
//!
//! `netlink-tc` provides a pure Rust API for interacting with the [netlink](https://www.kernel.org/doc/html/latest/userspace-api/netlink/intro.html) based Linux Traffic Control ([`tc`](http://man7.org/linux/man-pages/man8/tc.8.html)) subsystem of [`rtnetlink`](http://man7.org/linux/man-pages/man7/rtnetlink.7.html).
//!
//! This library is very much in progress. It only supports a small subset of `classless` and `classful` [qdiscs](https://tldp.org/HOWTO/Traffic-Control-HOWTO/components.html#c-qdisc). Also, the library only supports read at the moment.
//!
//! ## Example
//!
//! ```rust
//! use netlink_tc as tc;
//!
//! fn main() {
//!     // Get list of qdiscs
//!     let qdiscs = tc::qdiscs().unwrap();
//!
//!     // Get list of classes
//!     let classes = tc::classes().unwrap();
//!
//!     // Get class for given interface
//!     let class = tc::class("eth0").unwrap();
//! }
//! ```

pub mod link;
pub mod tc;
pub mod types;

mod class;
mod constants;
mod errors;
mod netlink;
mod qdiscs;

#[cfg(test)]
mod test_data;
#[cfg(test)]
mod tests;

pub use class::*;
pub use netlink::*;
pub use qdiscs::*;
pub use types::*;

/// Get list of all `tc` qdiscs and classes.
pub fn tc_stats() -> Result<Vec<Tc>, errors::TcError> {
    nl_tc_stats::<Netlink>()
}

/// Get list of all `tc` qdiscs.
pub fn qdiscs() -> Result<Vec<Tc>, errors::TcError> {
    nl_qdiscs::<Netlink>()
}

/// Get list of all `tc` classes.
pub fn classes() -> Result<Vec<Tc>, errors::TcError> {
    nl_classes::<Netlink>()
}

/// Get list of all `tc` classes for a given interface.
pub fn class(name: &str) -> Result<Vec<Tc>, errors::TcError> {
    nl_class::<Netlink>(name)
}

/// Get list of all `tc` classes for a given interface index.
pub fn class_for_index(index: u32) -> Result<Vec<Tc>, errors::TcError> {
    nl_class_for_index::<Netlink>(index)
}

fn nl_tc_stats<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, errors::TcError> {
    tc::tc_stats::<T>()
}

fn nl_qdiscs<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, errors::TcError> {
    tc::qdiscs::<T>()
}

fn nl_class<T: netlink::NetlinkConnection>(name: &str) -> Result<Vec<Tc>, errors::TcError> {
    tc::class::<T>(name)
}

fn nl_class_for_index<T: netlink::NetlinkConnection>(
    index: u32,
) -> Result<Vec<Tc>, errors::TcError> {
    tc::class_for_index::<T>(index)
}

fn nl_classes<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, errors::TcError> {
    tc::classes::<T>()
}
