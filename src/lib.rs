mod constants;
mod errors;
mod netlink;
mod qdiscs;
mod tc;

use std::collections::BTreeMap;

use tc::Tc;

pub fn tc_stats() -> Result<BTreeMap<u32, Vec<Tc>>, errors::TcError> {
    tc::tc_stats()
}
