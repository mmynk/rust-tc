# rust-tc

`rust-tc` provides a pure Rust API for interacting with the [netlink](https://www.kernel.org/doc/html/latest/userspace-api/netlink/intro.html) based Linux Traffic Control ([`tc`](http://man7.org/linux/man-pages/man8/tc.8.html)) subsystem of [`rtnetlink`](http://man7.org/linux/man-pages/man7/rtnetlink.7.html).

This library is very much in progress. It only supports a small subset of `classless` and `classful` [qdiscs](https://tldp.org/HOWTO/Traffic-Control-HOWTO/components.html#c-qdisc). Also, the library only supports read at the moment.

## Usage

```rust
use netlink_tc as tc;

fn main() {
    // Get list of qdiscs
    let qdiscs = tc::qdiscs::<tc::Netlink>().unwrap();

    // Get list of classes
    let classes = tc::classes::<tc::Netlink>().unwrap();

    // Get class for given interface
    let class = tc::class::<tc::Netlink>("eth0").unwrap();
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
