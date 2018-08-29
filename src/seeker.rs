use fst::{self, IntoStreamer, Map};
use fst_regex::Regex;

/// DocItem represent a searchable item
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DocItem {
    /// Relative url for the page of this item
    pub url: String,
    /// Description of this item
    pub desc: String,
}

impl DocItem {
    pub fn new(url: String, desc: String) -> DocItem {
        DocItem { url, desc }
    }
}

/// RustDoc contains DocItems, which could be convert to RustDocSeeker
///
/// # Example
///
/// ```
/// let data = fs::read_to_string("search-index.js").unwrap();
/// let rustdoc: RustDoc = data.parse().unwrap();
/// ```
#[derive(Debug)]
pub struct RustDoc {
    items: Vec<DocItem>,
}

/// RustDocSeeker contains DocItems and Index for fast searching
///
/// # Example
///
/// ```
/// let seeker = rustdoc.build().unwrap();
/// ```
#[derive(Debug)]
pub struct RustDocSeeker {
    items: Vec<DocItem>,
    index: Map,
}

impl RustDoc {
    pub fn new(items: Vec<DocItem>) -> RustDoc {
        RustDoc { items }
    }

    /// Build an index for searching
    pub fn build(self) -> Result<RustDocSeeker, fst::Error> {
        let mut items = self.items;
        items.sort_unstable_by(|a, b| a.url.cmp(&b.url));
        items.dedup_by(|a, b| a.url == b.url);

        let index = Map::from_iter(
            items
                .iter()
                .enumerate()
                .map(|(idx, item)| (&item.url, idx as u64)),
        )?;

        Ok(RustDocSeeker { items, index })
    }
}

impl RustDocSeeker {
    /// Regex based searching
    ///
    /// # Example
    ///
    /// ```
    /// for i in seeker.search(".*dedup.*") {
    ///     println!("{:?}", i);
    /// }
    /// ```
    pub fn search(&self, keyword: &str) -> impl Iterator<Item = &DocItem> {
        let lev = Regex::new(keyword).unwrap();
        let result = self.index.search(&lev).into_stream().into_values();

        result.into_iter().map(move |idx| &self.items[idx as usize])
    }
}
