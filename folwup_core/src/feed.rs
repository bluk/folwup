use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Feed<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub home_page_url: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item<'a> {
    pub id: Option<&'a str>,
    pub title: Option<&'a str>,
    pub content: Option<&'a str>,
    pub summary: Option<&'a str>,
    pub url: Option<&'a str>,
    pub external_url: Option<&'a str>,
    pub published_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct Iter<T> {
    feed: T,
}

#[derive(Debug)]
enum IterFeedTy<'a> {
    Atom(Iter<readfeed::atom::FeedIter<'a>>),
    Rss(Iter<readfeed::rss::ChannelIter<'a>>),
}

pub mod atom;
pub mod rss;

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn parse_feed(input: &str) -> Option<Feed<'_>> {
    match readfeed::detect_type(input) {
        readfeed::Ty::Atom => atom::parse_feed(input),
        readfeed::Ty::Rss => rss::parse_feed(input),
        readfeed::Ty::Json | readfeed::Ty::Unknown | readfeed::Ty::XmlOrHtml => None,
    }
}

impl<'a> Iter<IterFeedTy<'a>> {
    #[must_use]
    pub fn with_str(input: &'a str) -> Option<Self> {
        let feed = match readfeed::detect_type(input) {
            readfeed::Ty::Atom => IterFeedTy::Atom(Iter::with_atom(input)?),
            readfeed::Ty::Rss => IterFeedTy::Rss(Iter::with_rss(input)?),
            readfeed::Ty::Json | readfeed::Ty::Unknown | readfeed::Ty::XmlOrHtml => return None,
        };

        Some(Self { feed })
    }
}

impl<'a> Iterator for Iter<IterFeedTy<'a>> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.feed {
            IterFeedTy::Atom(iter) => iter.next(),
            IterFeedTy::Rss(iter) => iter.next(),
        }
    }
}
