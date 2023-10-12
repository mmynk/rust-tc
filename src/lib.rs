mod constants;
mod errors;
mod netlink;
mod qdiscs;
mod tc;
mod types;

pub use qdiscs::*;
pub use types::*;

pub fn tc_stats() -> Result<Vec<Tc>, errors::TcError> {
    tc::tc_stats()
}
