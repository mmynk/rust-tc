use tc::{classes, link, qdiscs, Netlink};

#[test]
fn test_get_qdiscs() {
    let result = qdiscs();
    assert!(result.is_ok());
    let tcs = result.unwrap();
    for tc in tcs {
        let attr = tc.attr;
        assert!(attr.kind.is_some());
        assert!(attr.stats.is_some());
        assert!(attr.stats2.is_some());
    }
}

#[test]
fn test_get_classes() {
    let result = classes();
    assert!(result.is_ok());
}

#[test]
fn test_get_links() {
    let result = link::links::<Netlink>();
    assert!(result.is_ok());
}
