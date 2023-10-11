use rust_tc::tc_stats;

#[test]
fn test_get_qdiscs() {
    let result = tc_stats();
    assert!(result.is_ok());
    let tc_map = result.unwrap();
    for (_, tcs) in tc_map {
        for tc in tcs {
            assert!(tc.kind.is_some());
            assert!(tc.stats.is_some());
        }
    }
}
