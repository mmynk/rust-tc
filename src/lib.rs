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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_qdiscs() {
        let result = tc_stats();
        assert!(result.is_ok());
    }
}
