use chrono::{DateTime, Utc};
use readfeed::atom;

use super::{Feed, Item, Iter};

impl<'a> Iter<atom::FeedIter<'a>> {
    #[must_use]
    pub fn with_atom(input: &'a str) -> Option<Self> {
        let xml = atom::Iter::new(input);
        for item in xml {
            match item {
                atom::Elem::Feed(feed) => return Some(Self { feed }),
                atom::Elem::Unknown(_) | atom::Elem::Raw(_) => {}
            }
        }

        None
    }
}

impl<'a> Iterator for Iter<atom::FeedIter<'a>> {
    type Item = Item<'a>;

    #[allow(clippy::too_many_lines)]
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

        fn capture_internal_link<'a>(
            existing_link: atom::Link<'a>,
            new_link: atom::Link<'a>,
        ) -> atom::Link<'a> {
            if let Some(rel) = new_link.rel().map(|v| v.as_str()) {
                let Some(existing_rel) = existing_link.rel().map(|r| r.as_str()) else {
                    if rel.eq_ignore_ascii_case("self") {
                        return new_link;
                    }
                    return existing_link;
                };

                macro_rules! check_rel_and_ty {
                    ($rel:literal) => {
                        if existing_rel.eq_ignore_ascii_case($rel) {
                            if rel.eq_ignore_ascii_case($rel) {
                                if existing_link.ty().is_some() {
                                    return existing_link;
                                }
                                let Some(ty) = new_link.ty().map(|v| v.as_str()) else {
                                    return existing_link;
                                };
                                if ty.eq_ignore_ascii_case("text/html") {
                                    return new_link;
                                }
                            }
                            return existing_link;
                        } else if rel.eq_ignore_ascii_case($rel) {
                            return new_link;
                        }
                    };
                }

                check_rel_and_ty!("self");
                check_rel_and_ty!("related");
                check_rel_and_ty!("alternate");
            } else if let Some(existing_rel) = existing_link.rel().map(|r| r.as_str()) {
                if existing_rel.eq_ignore_ascii_case("related")
                    || existing_rel.eq_ignore_ascii_case("alternate")
                {
                    return new_link;
                }
            } else {
                if existing_link.ty().is_some() {
                    return existing_link;
                }
                let Some(ty) = new_link.ty().map(|v| v.as_str()) else {
                    return existing_link;
                };
                if ty.eq_ignore_ascii_case("text/html") {
                    return new_link;
                }
            }

            existing_link
        }

        fn capture_external_link<'a>(
            existing_link: atom::Link<'a>,
            new_link: atom::Link<'a>,
        ) -> atom::Link<'a> {
            if let Some(rel) = new_link.rel().map(|v| v.as_str()) {
                let Some(existing_rel) = existing_link.rel().map(|r| r.as_str()) else {
                    if rel.eq_ignore_ascii_case("via")
                        || rel.eq_ignore_ascii_case("related")
                        || rel.eq_ignore_ascii_case("alternate")
                    {
                        return new_link;
                    }
                    return existing_link;
                };

                macro_rules! check_rel_and_ty {
                    ($rel:literal) => {
                        if existing_rel.eq_ignore_ascii_case($rel) {
                            if rel.eq_ignore_ascii_case($rel) {
                                if existing_link.ty().is_some() {
                                    return existing_link;
                                }
                                let Some(ty) = new_link.ty().map(|v| v.as_str()) else {
                                    return existing_link;
                                };
                                if ty.eq_ignore_ascii_case("text/html") {
                                    return new_link;
                                }
                            }
                            return existing_link;
                        } else if rel.eq_ignore_ascii_case($rel) {
                            return new_link;
                        }
                    };
                }

                check_rel_and_ty!("via");
                check_rel_and_ty!("related");
                check_rel_and_ty!("alternate");
                check_rel_and_ty!("self");
            } else if let Some(existing_rel) = existing_link.rel().map(|r| r.as_str()) {
                if existing_rel.eq_ignore_ascii_case("self") {
                    return new_link;
                }
            } else {
                if existing_link.ty().is_some() {
                    return existing_link;
                }
                let Some(ty) = new_link.ty().map(|v| v.as_str()) else {
                    return existing_link;
                };
                if ty.eq_ignore_ascii_case("text/html") {
                    return new_link;
                }
            }

            existing_link
        }

        for elem in self.feed.by_ref() {
            match elem {
                atom::FeedElem::Entry(entry_iter) => {
                    let mut item = Item::default();
                    let mut existing_internal_link: Option<atom::Link<'a>> = None;
                    let mut existing_external_link: Option<atom::Link<'a>> = None;

                    for entry_elem in entry_iter {
                        match entry_elem {
                            atom::EntryElem::Content(content) => {
                                item.content = item.content.or(Some(content.content()));
                            }
                            atom::EntryElem::Id(id) => {
                                item.id = item.id.or(Some(id.content()));
                            }
                            atom::EntryElem::Summary(summary) => {
                                item.summary = item.summary.or(Some(summary.content()));
                            }
                            atom::EntryElem::Title(title) => {
                                item.title = item.title.or(Some(title.content()));
                            }
                            atom::EntryElem::Updated(updated) => {
                                item.modified_at = item
                                    .modified_at
                                    .or_else(|| convert_datetime(updated.content()));
                            }
                            atom::EntryElem::Link(link) => {
                                if let Some(existing_link) = existing_internal_link {
                                    existing_internal_link =
                                        Some(capture_internal_link(existing_link, link));
                                } else {
                                    existing_internal_link = Some(link);
                                }

                                if let Some(existing_link) = existing_external_link {
                                    existing_external_link =
                                        Some(capture_external_link(existing_link, link));
                                } else {
                                    existing_external_link = Some(link);
                                }
                            }
                            atom::EntryElem::Published(published) => {
                                item.published_at = item
                                    .published_at
                                    .or_else(|| convert_datetime(published.content()));
                            }
                            atom::EntryElem::Source(source_iter) => {
                                for source_elem in source_iter {
                                    match source_elem {
                                        atom::SourceElem::Link(link) => {
                                            if let Some(existing_link) = existing_external_link {
                                                existing_external_link = Some(
                                                    capture_external_link(existing_link, link),
                                                );
                                            } else {
                                                existing_external_link = Some(link);
                                            }
                                        }
                                        atom::SourceElem::Author(_)
                                        | atom::SourceElem::Category(_)
                                        | atom::SourceElem::Contributor(_)
                                        | atom::SourceElem::Generator(_)
                                        | atom::SourceElem::Icon(_)
                                        | atom::SourceElem::Id(_)
                                        | atom::SourceElem::Logo(_)
                                        | atom::SourceElem::Rights(_)
                                        | atom::SourceElem::Subtitle(_)
                                        | atom::SourceElem::Title(_)
                                        | atom::SourceElem::Updated(_)
                                        | atom::SourceElem::Unknown(_)
                                        | atom::SourceElem::Raw(_) => {}
                                    }
                                }
                            }
                            atom::EntryElem::Author(_)
                            | atom::EntryElem::Category(_)
                            | atom::EntryElem::Contributor(_)
                            | atom::EntryElem::Rights(_)
                            | atom::EntryElem::Unknown(_)
                            | atom::EntryElem::Raw(_) => {}
                        }
                    }

                    item.url = existing_internal_link
                        .and_then(|l| l.href())
                        .map(|v| v.as_str());
                    item.external_url = existing_external_link
                        .and_then(|l| l.href())
                        .map(|v| v.as_str());

                    return Some(item);
                }
                atom::FeedElem::Id(_)
                | atom::FeedElem::Link(_)
                | atom::FeedElem::Subtitle(_)
                | atom::FeedElem::Title(_)
                | atom::FeedElem::Updated(_)
                | atom::FeedElem::Author(_)
                | atom::FeedElem::Category(_)
                | atom::FeedElem::Contributor(_)
                | atom::FeedElem::Generator(_)
                | atom::FeedElem::Icon(_)
                | atom::FeedElem::Logo(_)
                | atom::FeedElem::Rights(_)
                | atom::FeedElem::Unknown(_)
                | atom::FeedElem::Raw(_) => {}
            }
        }

        None
    }
}

#[must_use]
pub fn parse_feed(input: &str) -> Option<Feed<'_>> {
    fn capture_internal_link<'a>(
        existing_link: atom::Link<'a>,
        new_link: atom::Link<'a>,
    ) -> atom::Link<'a> {
        if let Some(rel) = new_link.rel().map(|v| v.as_str()) {
            let Some(existing_rel) = existing_link.rel().map(|r| r.as_str()) else {
                if rel.eq_ignore_ascii_case("self") {
                    return new_link;
                }
                return existing_link;
            };

            macro_rules! check_rel_and_ty {
                ($rel:literal) => {
                    if existing_rel.eq_ignore_ascii_case($rel) {
                        if rel.eq_ignore_ascii_case($rel) {
                            if existing_link.ty().is_some() {
                                return existing_link;
                            }
                            let Some(ty) = new_link.ty().map(|v| v.as_str()) else {
                                return existing_link;
                            };
                            if ty.eq_ignore_ascii_case("text/html") {
                                return new_link;
                            }
                        }
                        return existing_link;
                    } else if rel.eq_ignore_ascii_case($rel) {
                        return new_link;
                    }
                };
            }

            check_rel_and_ty!("self");
            check_rel_and_ty!("related");
            check_rel_and_ty!("alternate");
        } else if let Some(existing_rel) = existing_link.rel().map(|r| r.as_str()) {
            if existing_rel.eq_ignore_ascii_case("related")
                || existing_rel.eq_ignore_ascii_case("alternate")
            {
                return new_link;
            }
        } else {
            if existing_link.ty().is_some() {
                return existing_link;
            }
            let Some(ty) = new_link.ty().map(|v| v.as_str()) else {
                return existing_link;
            };
            if ty.eq_ignore_ascii_case("text/html") {
                return new_link;
            }
        }

        existing_link
    }

    let xml = atom::Iter::new(input);
    for item in xml {
        match item {
            atom::Elem::Feed(feed_iter) => {
                let mut feed = Feed::default();
                let mut existing_internal_link: Option<atom::Link<'_>> = None;
                for feed_elem in feed_iter {
                    match feed_elem {
                        atom::FeedElem::Title(title) => {
                            feed.title = feed.title.or(Some(title.content()));
                        }
                        atom::FeedElem::Subtitle(subtitle) => {
                            feed.description = feed.description.or(Some(subtitle.content()));
                        }
                        atom::FeedElem::Link(link) => {
                            if let Some(existing_link) = existing_internal_link {
                                existing_internal_link =
                                    Some(capture_internal_link(existing_link, link));
                            } else {
                                existing_internal_link = Some(link);
                            }
                        }
                        atom::FeedElem::Author(_)
                        | atom::FeedElem::Category(_)
                        | atom::FeedElem::Contributor(_)
                        | atom::FeedElem::Generator(_)
                        | atom::FeedElem::Icon(_)
                        | atom::FeedElem::Id(_)
                        | atom::FeedElem::Logo(_)
                        | atom::FeedElem::Rights(_)
                        | atom::FeedElem::Updated(_)
                        | atom::FeedElem::Entry(_)
                        | atom::FeedElem::Unknown(_)
                        | atom::FeedElem::Raw(_) => {}
                    }
                }

                return Some(feed);
            }
            atom::Elem::Unknown(_) | atom::Elem::Raw(_) => {}
        }
    }

    None
}
