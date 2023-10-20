use netlink_packet_route::nlas::link::Nla;

use crate::{netlink, Link, errors::LinkError};

/// `links` parses `LinkMessage`s into `Link`s and returns them.
pub fn links<T: netlink::NetlinkConnection>() -> Result<Vec<Link>, LinkError> {
    let mut links = Vec::new();

    let messages = T::new()?.links()?;
    for message in messages {
        let index = message.header.index;
        let mut name = None;

        for attr in message.nlas {
            match attr {
                Nla::IfName(ifname) => {
                    name = Some(ifname);
                }
                _ => (),
            }
        }

        if name.is_none() {
            return Err(LinkError::MissingAttribute("Interface name not found".to_string()));
        }


        links.push(Link {
            index,
            name: name.unwrap(),
        });
    }

    Ok(links)
}
