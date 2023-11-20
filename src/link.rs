use crate::{errors::Error, Link, LinkMsg, RtNetlinkMessage};

/// `links` parses intermediate representation of netlink link messages `LinkMsg`s into `Link`s.
pub fn links(messages: Vec<RtNetlinkMessage>) -> Result<Vec<Link>, Error> {
    let messages = messages
        .into_iter()
        .filter_map(|message| match message {
            RtNetlinkMessage::GetLink(message) => Some(message),
            _ => None,
        })
        .collect::<Vec<LinkMsg>>();
    let mut links = Vec::with_capacity(messages.len());

    for message in messages {
        links.push(Link {
            index: message.header.index,
            name: message.attr.name,
        });
    }

    Ok(links)
}
