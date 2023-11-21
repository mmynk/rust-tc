# rust-tc

`rust-tc` provides a pure Rust API for interacting with the [netlink](https://www.kernel.org/doc/html/latest/userspace-api/netlink/intro.html) based Linux Traffic Control ([`tc`](http://man7.org/linux/man-pages/man8/tc.8.html)) subsystem of [`rtnetlink`](http://man7.org/linux/man-pages/man7/rtnetlink.7.html).

This library is very much in progress. It only supports a small subset of `classless` and `classful` [qdiscs](https://tldp.org/HOWTO/Traffic-Control-HOWTO/components.html#c-qdisc). Also, the library only supports read at the moment.

## Usage

```rust
use netlink_packet_core::NetlinkMessage;
use netlink_packet_route::RtnlMessage;
use netlink_tc as tc;

fn main() {
    // Retrive netlink messages using `netlink-packet-route`.
    // See `examples` for more details.
    let messages: Vec<NetlinkMessage<RtnlMessage>> = vec![]; // init with netlink messages

    // Get list of tc qdiscs or classes
    let qdiscs = OpenOptions::new()
        .fail_on_unknown_netlink_message(true)
        .tc(messages.clone()).unwrap();

    // Get list of links
    let links = OpenOptions::new()
        .links(messages.clone()).unwrap();
}
```

## TODO
* Add support for all qdiscs and classes.
* Add support for write, update and delete.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
