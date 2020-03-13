use crate::{
    json::fix_json,
    seeker::{DocItem, RustDoc, TypeItem},
};
use serde::Deserialize;
use serde_json::{self, Value};
use std::{collections::BTreeSet, str::FromStr};
use string_cache::DefaultAtom as Atom;

#[derive(Clone, Debug, Deserialize)]
struct Parent {
    ty: usize,
    name: Atom,
}

#[derive(Debug, Deserialize)]
struct IndexItem {
    ty: usize,
    name: Atom,
    path: Atom,
    desc: Atom,
    #[serde(skip_deserializing)]
    parent: Option<Parent>,
    parent_idx: Option<usize>,
    search_type: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct SearchIndex {
    doc: Atom,
    #[serde(rename = "i")]
    items: Vec<IndexItem>,
    #[serde(rename = "p")]
    paths: Vec<Parent>,
}

impl From<IndexItem> for DocItem {
    /// Convert an IndexItem to DocItem based on if parent exists.
    fn from(item: IndexItem) -> DocItem {
        let name = TypeItem::new(item.ty, item.name);
        let parent = item.parent.map(|x| TypeItem::new(x.ty, x.name));

        DocItem::new(name, parent, item.path, item.desc)
    }
}

impl FromStr for RustDoc {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = BTreeSet::new();

        for line in s.lines().filter(|x| x.starts_with("searchIndex")) {
            let eq = line.find('=').unwrap() + 1;
            let line = line.split_at(eq).1.trim().trim_end_matches(';');

            let json = fix_json(line);

            let index: SearchIndex = serde_json::from_str(&json).unwrap();

            let mut last_path = Atom::from("");
            let parents = index.paths;

            for mut item in index.items {
                // if `path` is empty, the `path` is the same as previous one
                // Dirty trick to compress the file size
                if !item.path.is_empty() {
                    last_path = item.path;
                };

                item.path = last_path.clone();

                // parent_idx is the index of the item in SearchIndex.paths
                item.parent = item.parent_idx.map(|idx| parents[idx].clone());

                items.insert(DocItem::from(item));
            }
        }

        Ok(RustDoc::new(items))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn test_parser() {
        let data = fs::read_to_string("search-index.js").unwrap();
        let _: RustDoc = data.parse().unwrap();
    }
}
