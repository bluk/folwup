use chrono::{DateTime, Utc};
use readfeed::rss;

use super::{Feed, Item, Iter};

#[must_use]
pub fn parse_feed(input: &str) -> Option<Feed<'_>> {
    let xml = rss::Iter::new(input);

    for item in xml {
        match item {
            rss::Elem::Rss(rss_iter) => {
                for rss_item in rss_iter {
                    match rss_item {
                        rss::RssElem::Channel(channel_iter) => {
                            let mut feed = Feed::default();

                            for elem in channel_iter {
                                match elem {
                                    rss::ChannelElem::Title(title) => {
                                        feed.title = feed.title.or(Some(title.content()));
                                    }
                                    rss::ChannelElem::Description(desc) => {
                                        feed.description =
                                            feed.description.or(Some(desc.content()));
                                    }
                                    rss::ChannelElem::Link(link) => {
                                        feed.home_page_url =
                                            feed.home_page_url.or(Some(link.content()));
                                    }
                                    rss::ChannelElem::PubDate(_)
                                    | rss::ChannelElem::LastBuildDate(_)
                                    | rss::ChannelElem::Image(_)
                                    | rss::ChannelElem::Item(_)
                                    | rss::ChannelElem::Language(_)
                                    | rss::ChannelElem::Copyright(_)
                                    | rss::ChannelElem::ManagingEditor(_)
                                    | rss::ChannelElem::Webmaster(_)
                                    | rss::ChannelElem::Category(_)
                                    | rss::ChannelElem::Generator(_)
                                    | rss::ChannelElem::Docs(_)
                                    | rss::ChannelElem::Ttl(_)
                                    | rss::ChannelElem::Rating(_)
                                    | rss::ChannelElem::SkipHours(_)
                                    | rss::ChannelElem::SkipDays(_)
                                    | rss::ChannelElem::Unknown(_)
                                    | rss::ChannelElem::Raw(_) => {}
                                }
                            }

                            return Some(feed);
                        }
                        rss::RssElem::Unknown(_) | rss::RssElem::Raw(_) => {}
                    }
                }
            }
            rss::Elem::Unknown(_) | rss::Elem::Raw(_) => {}
        }
    }

    None
}

impl<'a> Iter<rss::ChannelIter<'a>> {
    #[must_use]
    pub fn with_rss(input: &'a str) -> Option<Self> {
        let xml = rss::Iter::new(input);

        for item in xml {
            match item {
                rss::Elem::Rss(rss_iter) => {
                    for rss_item in rss_iter {
                        match rss_item {
                            rss::RssElem::Channel(feed) => return Some(Self { feed }),
                            rss::RssElem::Unknown(_) | rss::RssElem::Raw(_) => {}
                        }
                    }
                }
                rss::Elem::Unknown(_) | rss::Elem::Raw(_) => {}
            }
        }

        None
    }
}

impl<'a> Iterator for Iter<rss::ChannelIter<'a>> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        fn convert_datetime(datetime: &str) -> Option<DateTime<Utc>> {
            if let Ok(updated) = datetime.parse::<DateTime<Utc>>() {
                return Some(updated);
            }

            if let Ok(updated) = DateTime::parse_from_rfc3339(datetime) {
                return Some(updated.into());
            }

            if let Ok(updated) = DateTime::parse_from_rfc2822(datetime) {
                return Some(updated.into());
            }

            None
        }

        for elem in self.feed.by_ref() {
            match elem {
                rss::ChannelElem::Item(item_iter) => {
                    let mut item = Item::default();

                    for item_elem in item_iter {
                        match item_elem {
                            rss::ItemElem::Title(title) => {
                                item.title = item.title.or(Some(title.content()));
                            }
                            rss::ItemElem::Link(link) => {
                                item.url = item.url.or(Some(link.content()));
                            }
                            rss::ItemElem::Description(description) => {
                                item.summary = item.summary.or(Some(description.content()));
                            }
                            rss::ItemElem::Guid(guid) => {
                                item.id = item.id.or(Some(guid.content()));
                            }
                            rss::ItemElem::PubDate(published) => {
                                item.published_at = item
                                    .published_at
                                    .or_else(|| convert_datetime(published.content()));
                            }
                            rss::ItemElem::Author(_)
                            | rss::ItemElem::Category(_)
                            | rss::ItemElem::Enclosure(_)
                            | rss::ItemElem::Source(_)
                            | rss::ItemElem::Comments(_)
                            | rss::ItemElem::Unknown(_)
                            | rss::ItemElem::Raw(_) => {}
                        }
                    }

                    return Some(item);
                }
                rss::ChannelElem::Title(_)
                | rss::ChannelElem::Link(_)
                | rss::ChannelElem::Description(_)
                | rss::ChannelElem::PubDate(_)
                | rss::ChannelElem::LastBuildDate(_)
                | rss::ChannelElem::Image(_)
                | rss::ChannelElem::Language(_)
                | rss::ChannelElem::Copyright(_)
                | rss::ChannelElem::ManagingEditor(_)
                | rss::ChannelElem::Webmaster(_)
                | rss::ChannelElem::Category(_)
                | rss::ChannelElem::Generator(_)
                | rss::ChannelElem::Docs(_)
                | rss::ChannelElem::Ttl(_)
                | rss::ChannelElem::Rating(_)
                | rss::ChannelElem::SkipHours(_)
                | rss::ChannelElem::SkipDays(_)
                | rss::ChannelElem::Unknown(_)
                | rss::ChannelElem::Raw(_) => {}
            }
        }

        None
    }
}
