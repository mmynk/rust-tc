mod class;
mod constants;
mod errors;
mod link;
mod netlink;
mod qdiscs;
mod tc;
mod types;

#[cfg(test)]
mod tests;

pub use class::*;
pub use link::*;
pub use netlink::*;
pub use qdiscs::*;
pub use tc::*;
pub use types::*;

pub fn tc_stats<T: netlink::NetlinkConnection>() -> Result<Vec<Tc>, errors::TcError> {
    tc::tc_stats::<T>()
}
