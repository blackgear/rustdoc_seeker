use fst::{Automaton, IntoStreamer, Map, MapBuilder};
use itertools::Itertools;
use std::{
    cmp::{Ord, Ordering},
    collections::BTreeSet,
    fmt,
    iter::FromIterator,
    u32,
};
use string_cache::DefaultAtom as Atom;

macro_rules! enum_number {
    ($name:ident { $($variant:ident | $display:tt | $value:tt, )* }) => {
        /// TypeItem represent an item with type,
        /// Use `Display` or `fmt_url` to get the `type dot name` format of the item.
        ///
        /// # Example
        ///
        /// ```
        /// assert_eq!("module.vec", TypeItme::Module("vec"));
        /// assert_eq!("macro.vec", TypeItme::Macro("vec"));
        ///
        /// assert_eq!("fn.vec", TypeItme::Function("vec")); // the only two exceptions
        /// assert_eq!("type.vec", TypeItme::Typedef("vec")); // the only two exceptions
        /// ```
        #[derive(Clone, Debug, Eq, PartialEq)]
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
        }

        impl AsRef<Atom> for $name {
            fn as_ref(&self) -> &Atom {
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
    Function        | "fn"              | 5,
    Typedef         | "type"            | 6,
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

/// DocItem represent a searchable item,
/// Use `Display` to get the relative URI of the item.
///
/// eg:
///
/// The `dedup` (name) function of the `Vec`(parent) struct in `std::vec`(path) module.
///
/// The `Vec`(name) struct of `None`(parent) in `std::vec`(path) module.
///
/// The `vec`(name) module of `None`(parent) in `std`(path) module.
///
/// # Example
///
/// ```
/// println!("{} is the url of {:?}", &docitem, &docitem)
/// ```
#[derive(Debug, Eq)]
pub struct DocItem {
    pub name: TypeItem,
    pub parent: Option<TypeItem>,
    pub path: Atom,
    pub desc: Atom,
}

impl DocItem {
    pub fn new(name: TypeItem, parent: Option<TypeItem>, path: Atom, desc: Atom) -> DocItem {
        DocItem {
            name,
            parent,
            path,
            desc,
        }
    }

    pub fn fmt_naive<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        write!(f, "{}::", self.path)?;
        if let Some(ref parent) = self.parent {
            write!(f, "{}::", parent)?;
        };
        write!(f, "{}", self.name)
    }

    pub fn fmt_url<W: fmt::Write>(&self, f: &mut W) -> fmt::Result {
        for part in self.path.split("::") {
            write!(f, "{}/", part)?;
        }
        if let Some(ref parent) = self.parent {
            write!(f, "{}.html#{}", parent, self.name)
        } else {
            match &self.name {
                TypeItem::Module(name) => write!(f, "{}/index.html", name),
                _ => write!(f, "{}.html", self.name),
            }
        }
    }

    fn parent_atom(&self) -> Option<&Atom> {
        self.parent.as_ref().map(|p| p.as_ref())
    }

    fn index_key(&self) -> &[u8] {
        self.name.as_ref().as_bytes()
    }
}

impl PartialEq for DocItem {
    fn eq(&self, other: &DocItem) -> bool {
        self.name == other.name && self.parent == other.parent && self.path == other.path
    }
}

impl Ord for DocItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index_key()
            .cmp(&other.index_key())
            .then_with(|| self.path.cmp(&other.path))
            .then_with(|| self.parent_atom().cmp(&other.parent_atom()))
    }
}

impl PartialOrd for DocItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DocItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_url(f)
    }
}

/// RustDoc contains DocItems, which could be convert to RustDocSeeker.
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
    items: BTreeSet<DocItem>,
}

impl Extend<DocItem> for RustDoc {
    fn extend<T: IntoIterator<Item=DocItem>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<DocItem> for RustDoc {
    fn from_iter<I: IntoIterator<Item=DocItem>>(iter: I) -> Self {
        RustDoc {
            items: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for RustDoc {
    type IntoIter = <BTreeSet<DocItem> as IntoIterator>::IntoIter;
    type Item = DocItem;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl RustDoc {
    pub fn new(items: BTreeSet<DocItem>) -> RustDoc {
        RustDoc {
            items,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&DocItem> {
        self.items.iter()
    }

    /// Build an index for searching
    pub fn build(self) -> RustDocSeeker {
        let mut builder = MapBuilder::memory();
        let items = self.items.into_iter().collect_vec().into_boxed_slice();

        assert!(items.len() as u64 <= u32::MAX as u64);

        {
            let groups = items
                .iter()
                .enumerate()
                .group_by(|(_, item)| item.index_key());

            for (key, mut group) in groups.into_iter() {
                let (start, _) = group.next().unwrap();
                let end = group.last().map_or(start, |(i, _)| i) + 1;
                let val = ((start as u64) << 32) + end as u64;
                // We already sort and dedup using BTreeSet, so it always safe to unwrap.
                builder.insert(key, val).unwrap();
            }
        }

        let index = builder.into_map();
        RustDocSeeker {
            items,
            index,
        }
    }
}

/// RustDocSeeker contains DocItems and Index for fast searching.
///
/// The index is kv-map for <name, idx: u64 = (start: u32 << 32) + end: u32>
/// where items[start..end] having the same DocItem.name.
///
/// # Example
///
/// ```
/// let seeker = rustdoc.build();
/// ```
#[derive(Debug)]
pub struct RustDocSeeker {
    items: Box<[DocItem]>,
    index: Map<Vec<u8>>,
}

impl RustDocSeeker {
    /// Search with fst::Automaton, read fst::automaton / fst-levenshtein / fst-regex for details.
    ///
    /// # Example
    ///
    /// ```
    /// let aut = fst_regex::Regex::new(".*dedup.*").unwrap();
    /// for i in seeker.search(aut) {
    ///     println!("{:?}", i);
    /// }
    ///
    /// let aut = fst_levenshtein::Levenshtein::new("dedXp", 1).unwrap();
    /// for i in seeker.search(aut) {
    ///     println!("{:?}", i);
    /// }
    ///
    /// let aut = fst::automaton::Subsequence::new("dedup");
    /// for i in seeker.search(aut) {
    ///     println!("{:?}", i);
    /// }
    /// ```
    pub fn search<A: Automaton>(&self, aut: &A) -> impl Iterator<Item=&DocItem> {
        let result = self.index.search(aut).into_stream().into_values();

        result.into_iter().flat_map(move |idx| {
            let start = (idx >> 32) as usize;
            let end = (idx & 0xffffffff) as usize;
            &self.items[start..end]
        })
    }
}
