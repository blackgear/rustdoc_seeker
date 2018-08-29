//! # Example
//! ```
//! let data = fs::read_to_string("search-index.js").unwrap();
//! let rustdoc: RustDoc = data.parse().unwrap();
//! let seeker = rustdoc.build().unwrap();
//! for i in seeker.search(".*dedup.*") {
//!     println!("{:#?}", i);
//! }
//! ```

extern crate fst;
extern crate fst_regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate string_cache;

mod parser;
mod seeker;

pub use seeker::{DocItem, RustDoc, RustDocSeeker};
