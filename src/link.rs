use crate::{errors::LinkError, netlink, Link};

/// `links` parses intermediate representation of netlink link messages `LinkMsg`s into `Link`s.
pub fn links<T: netlink::NetlinkConnection>() -> Result<Vec<Link>, LinkError> {
    let mut links = Vec::new();

    let messages = T::new()?.links()?;
    for message in messages {
        links.push(Link {
            index: message.header.index,
            name: message.attr.name,
        });
    }

    Ok(links)
}
