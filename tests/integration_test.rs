use netlink_tc as tc;

#[test]
fn test_get_qdiscs() {
    let result = tc::qdiscs::<tc::Netlink>();
    assert!(result.is_ok());
    let tcs = result.unwrap();
    for tc in tcs {
        let attr = tc.attr;
        assert!(!attr.kind.is_empty());
        assert!(attr.stats.is_some());
        assert!(attr.stats2.is_some());
    }
}

#[test]
fn test_get_classes() {
    let result = tc::classes::<tc::Netlink>();
    assert!(result.is_ok());
}

#[test]
fn test_get_links() {
    let result = tc::link::links::<tc::Netlink>();
    assert!(result.is_ok());
}
