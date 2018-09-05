use fst::{self, Automaton, IntoStreamer, Map, MapBuilder};
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use string_cache::DefaultAtom as Atom;

macro_rules! enum_number {
    ($name:ident { $($variant:ident | $display:tt | $value:tt, )* }) => {
        /// TypeItem represent an item with type,
        /// Use `Display` to get the `type dot name` format of the item
        ///
        /// # Example
        /// ```
        /// assert_eq!("module.vec", TypeItme::Module(vec));
        /// assert_eq!("macro.vec", TypeItme::Macro(vec));
        ///
        /// assert_eq!("fn.vec", TypeItme::Function(vec)); // the only two exceptions
        /// assert_eq!("type.vec", TypeItme::Typedef(vec)); // the only two exceptions
        /// ```
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
/// Use `Display` to get the relative URI of the item
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

    /// Return the key of the DocItem for index
    pub fn key(&self) -> &[u8] {
        self.name.as_ref().as_bytes()
    }
}

impl PartialEq for DocItem {
    fn eq(&self, other: &DocItem) -> bool {
        self.name == other.name && self.parent == other.parent && self.path == other.path
    }
}

impl Hash for DocItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.parent.hash(state);
        self.path.hash(state);
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
            match &self.name {
                TypeItem::Module(name) => write!(f, "{}/index.html", name),
                _ => write!(f, "{}.html", self.name),
            }
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
                    builder.insert(name, ((start as u64) << 32) + idx as u64)?;
                    name = items[idx].key();
                    start = idx;
                };
            }

            builder.insert(name, ((start as u64) << 32) + items.len() as u64)?;
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
    ///
    /// let aut = fst::automaton::Subsequence::new("dedup", 1).unwrap();
    /// for i in seeker.search(aut) {
    ///     println!("{:?}", i);
    /// }
    ///
    /// ```
    pub fn search<A: Automaton>(&self, aut: &A) -> impl Iterator<Item = &DocItem> {
        let result = self.index.search(aut).into_stream().into_values();

        result.into_iter().flat_map(move |idx| {
            let start = (idx >> 32) as usize;
            let end = (idx & 0xffffffff) as usize;
            &self.items[start..end]
        })
    }
}
