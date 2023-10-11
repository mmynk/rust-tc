mod constants;
mod errors;
mod netlink;
mod qdiscs;
mod tc;
mod types;

use types::Tc;

pub fn tc_stats() -> Result<Vec<Tc>, errors::TcError> {
    tc::tc_stats()
}
