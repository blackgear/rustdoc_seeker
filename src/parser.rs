use seeker::{DocItem, RustDoc};
use serde_json;
use std::fmt::Write;
use std::str::FromStr;
use string_cache::DefaultAtom;

macro_rules! enum_number {
    ($name:ident { $($variant:ident|$display:expr ; $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                match self {
                    $( $name::$variant => write!(f, "{}", $display), )*
                }
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                    where
                        E: ::serde::de::Error,
                    {
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

enum_number!(ItemType {
    Module          | "module"          ; 0,
    ExternCrate     | "externcrate"     ; 1,
    Import          | "import"          ; 2,
    Struct          | "struct"          ; 3,
    Enum            | "enum"            ; 4,
    Function        | "function"        ; 5,
    Typedef         | "typedef"         ; 6,
    Static          | "static"          ; 7,
    Trait           | "trait"           ; 8,
    Impl            | "impl"            ; 9,
    TyMethod        | "tymethod"        ; 10,
    Method          | "method"          ; 11,
    StructField     | "structfield"     ; 12,
    Variant         | "variant"         ; 13,
    Macro           | "macro"           ; 14,
    Primitive       | "primitive"       ; 15,
    AssociatedType  | "associatedtype"  ; 16,
    Constant        | "constant"        ; 17,
    AssociatedConst | "associatedconst" ; 18,
    Union           | "union"           ; 19,
    ForeignType     | "foreigntype"     ; 20,
    Keyword         | "keyword"         ; 21,
    Existential     | "existential"     ; 22,
});

#[derive(Debug, Deserialize)]
struct IndexItemFunctionType {
    #[serde(rename = "i")]
    inputs: Option<Vec<Type>>,
    #[serde(rename = "o")]
    output: Option<Type>,
}

#[derive(Debug, Deserialize)]
struct Type {
    #[serde(rename = "n")]
    name: Option<DefaultAtom>,
    #[serde(rename = "g")]
    generics: Option<Vec<DefaultAtom>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Parent {
    ty: ItemType,
    name: DefaultAtom,
}

#[derive(Debug, Deserialize)]
struct IndexItem {
    ty: ItemType,
    name: DefaultAtom,
    path: DefaultAtom,
    desc: DefaultAtom,
    #[serde(skip_deserializing)]
    parent: Option<Parent>,
    parent_idx: Option<usize>,
    search_type: Option<IndexItemFunctionType>,
}

#[derive(Debug, Deserialize)]
struct SearchIndex {
    doc: DefaultAtom,
    items: Vec<IndexItem>,
    paths: Vec<Parent>,
}

impl From<IndexItem> for DocItem {
    /// Convert an IndexItem to DocItem based on if parent exists.
    fn from(item: IndexItem) -> DocItem {
        let mut url = item.path.replace("::", "/");
        if let Some(ref parent) = item.parent {
            write!(
                url,
                "/{}.{}.html#{}.{}",
                parent.ty, parent.name, item.ty, item.name
            )
        } else {
            write!(url, "/{}.{}.html", item.ty, item.name)
        }.unwrap();

        let desc = if let Some(ref parent) = item.parent {
            format!(
                "{}::{}::{} {}",
                item.path, parent.name, item.name, item.desc
            )
        } else {
            format!("{}::{} {}", item.path, item.name, item.desc)
        };

        DocItem::new(url, desc)
    }
}

impl FromStr for RustDoc {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut items = Vec::new();

        for line in s.lines().filter(|x| x.starts_with("searchIndex")) {
            let start = line.find('=').unwrap() + 2;
            let end = line.len() - 1;
            let index: SearchIndex = serde_json::from_str(&line[start..end]).unwrap();

            let mut last_path = DefaultAtom::from("");
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

                items.push(DocItem::from(item));
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
              null,
              null
            ],
            [
              8,
              "Any",
              "std::any",
              "A type to emulate dynamic typing.",
              null,
              null
            ],
            [
              10,
              "alloc",
              "",
              "Returns a pointer meeting the size and alignment guarantees of `layout`.",
              3,
              {"i":[{"n":"self"},{"n":"layout"}],"o":{"g":["nonnull","allocerr"],"n":"result"}}]
          ]
        }
        "#;
        let index: SearchIndex = serde_json::from_str(data).unwrap();
        println!("{:?}", index);
    }
}
