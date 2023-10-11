use rust_tc::tc_stats;

#[test]
fn test_get_qdiscs() {
    let result = tc_stats();
    assert!(result.is_ok());
    let tcs = result.unwrap();
    for tc in tcs {
        println!("tc: {:?}", tc);

        let attr = tc.attr;
        assert!(attr.kind.is_some());
        assert!(attr.stats.is_some());
        assert!(attr.stats2.is_some());
    }
}
