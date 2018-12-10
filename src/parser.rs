use json::fix_json;
use seeker::{DocItem, RustDoc, TypeItem};
use serde_json::{self, Value};
use std::collections::BTreeSet;
use std::str::FromStr;
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
    items: Vec<IndexItem>,
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

    #[test]
    fn test_parser() {
        let data = r#"
        {
          "doc": "The Rust Standard Library",
          "items": [
            [
              0,
              "any",
              "std",
              "This module implements the `Any` trait, which enables dynamic typing of any `'static` type through runtime reflection.",
              N,
              N
            ],
            [
              8,
              "Any",
              "std::any",
              "A type to emulate dynamic typing.",
              N,
              N
            ],
            [
              10,
              "alloc",
              "",
              "Returns a pointer meeting the size and alignment guarantees of `layout`.",
              3,
              {"i":[{"n":"self"},{"n":"layout"}],"o":{"g":["nonnull","allocerr"],"n":"result"}}
            ]
          ],
          "paths": [
            [0, ""],
            [0, ""],
            [8, "Alloc"]
          ]
        }
        "#;
        let index: SearchIndex = serde_json::from_str(data).unwrap();
        println!("{:?}", index);
    }
}
