use fst::{self, IntoStreamer, Map, MapBuilder};
use fst_levenshtein::Levenshtein;
use fst_regex::Regex;
use std::collections::HashSet;
use std::fmt;
use std::iter::FromIterator;
use string_cache::DefaultAtom as Atom;

macro_rules! enum_number {
    ($name:ident { $($variant:ident | $display:tt | $value:tt, )* }) => {
        /// TypeItem represent an item with type
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $($variant(Atom),)*
        }

        impl $name {
            pub fn new(tag: usize, data: Atom) -> $name {
                match tag {
                    $( $value => $name::$variant(data), )*
                    _ => unreachable!()
                }
            }

            pub fn plain(&self) -> &Atom {
                match self {
                    $( $name::$variant(data) => data, )*
                }

            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $( $name::$variant(data) => write!(f, "{}.{}", $display, data), )*
                }
            }
        }
    }
}

enum_number!(TypeItem {
    Module          | "module"          | 0,
    ExternCrate     | "externcrate"     | 1,
    Import          | "import"          | 2,
    Struct          | "struct"          | 3,
    Enum            | "enum"            | 4,
    Function        | "function"        | 5,
    Typedef         | "typedef"         | 6,
    Static          | "static"          | 7,
    Trait           | "trait"           | 8,
    Impl            | "impl"            | 9,
    TyMethod        | "tymethod"        | 10,
    Method          | "method"          | 11,
    StructField     | "structfield"     | 12,
    Variant         | "variant"         | 13,
    Macro           | "macro"           | 14,
    Primitive       | "primitive"       | 15,
    AssociatedType  | "associatedtype"  | 16,
    Constant        | "constant"        | 17,
    AssociatedConst | "associatedconst" | 18,
    Union           | "union"           | 19,
    ForeignType     | "foreigntype"     | 20,
    Keyword         | "keyword"         | 21,
    Existential     | "existential"     | 22,
});

/// DocItem represent a searchable item
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct DocItem {
    pub name: TypeItem,
    pub parent: Option<TypeItem>,
    pub path: Atom,
}

impl DocItem {
    pub fn new(name: TypeItem, parent: Option<TypeItem>, path: Atom) -> DocItem {
        DocItem { name, parent, path }
    }

    pub fn key(&self) -> &Atom {
        self.name.plain()
    }
}

impl fmt::Display for DocItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for part in self.path.split("::") {
            write!(f, "{}/", part)?;
        }
        if let Some(ref parent) = self.parent {
            write!(f, "{}.html#{}", parent, self.name)
        } else {
            write!(f, "{}.html", self.name)
        }
    }
}

/// RustDoc contains DocItems, which could be convert to RustDocSeeker
///
/// # Example
///
/// ```
/// let data = fs::read_to_string("search-index.js").unwrap();
/// let rustdoc: RustDoc = data.parse().unwrap();
///
/// // let's combine RustDoc
/// rustdoc_a.extend(rustdoc_b)
/// ```
#[derive(Debug)]
pub struct RustDoc {
    items: HashSet<DocItem>,
}

impl Extend<DocItem> for RustDoc {
    fn extend<T: IntoIterator<Item = DocItem>>(&mut self, iter: T) {
        for elem in iter {
            self.items.insert(elem);
        }
    }
}

impl FromIterator<DocItem> for RustDoc {
    fn from_iter<I: IntoIterator<Item = DocItem>>(iter: I) -> Self {
        RustDoc {
            items: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for RustDoc {
    type Item = DocItem;
    type IntoIter = ::std::collections::hash_set::IntoIter<DocItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl RustDoc {
    pub fn new(items: HashSet<DocItem>) -> RustDoc {
        RustDoc { items }
    }

    /// Build an index for searching
    pub fn build(self) -> Result<RustDocSeeker, fst::Error> {
        let mut builder = MapBuilder::memory();
        let mut items: Vec<_> = self.items.into_iter().collect();

        if items.len() > 0 {
            items.sort_unstable_by(|a, b| a.key().cmp(b.key()));
            let mut name = items[0].key();
            let mut start = 0;

            for idx in 1..items.len() {
                if name != items[idx].key() {
                    builder.insert(name.as_bytes(), ((start as u64) << 32) + idx as u64)?;
                    name = items[idx].key();
                    start = idx;
                };
            }

            builder.insert(name.as_bytes(), ((start as u64) << 32) + items.len() as u64)?;
        }

        let index = Map::from_bytes(builder.into_inner()?)?;
        Ok(RustDocSeeker { items, index })
    }
}

/// RustDocSeeker contains DocItems and Index for fast searching
///
/// The index is kv-map fro <name, idx: u64 = (start: u32 << 32) + end: u32>
/// where items[start..end] have the same DocItem.name.
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

impl RustDocSeeker {
    /// Regex based searching
    ///
    /// # Example
    ///
    /// ```
    /// for i in seeker.search_regex(".*dedup.*") {
    ///     println!("{:?}", i);
    /// }
    /// ```
    pub fn search_regex(&self, keyword: &str) -> impl Iterator<Item = &DocItem> {
        let dfa = Regex::new(keyword).unwrap();
        let result = self.index.search(&dfa).into_stream().into_values();

        result.into_iter().flat_map(move |idx| {
            let start = (idx >> 32) as usize;
            let end = (idx & 0xffffffff) as usize;
            &self.items[start..end]
        })
    }

    /// Edit Distence based searching
    ///
    /// # Example
    ///
    /// ```
    /// for i in seeker.search_edist("dedup", ("dedup".len() as f32 * 0.3) as u32) {
    ///     println!("{:?}", i);
    /// }
    /// ```
    pub fn search_edist(&self, keyword: &str, distance: u32) -> impl Iterator<Item = &DocItem> {
        let dfa = Levenshtein::new(keyword, distance).unwrap();
        let result = self.index.search(&dfa).into_stream().into_values();

        result.into_iter().flat_map(move |idx| {
            let start = (idx >> 32) as usize;
            let end = (idx & 0xffffffff) as usize;
            &self.items[start..end]
        })
    }
}
