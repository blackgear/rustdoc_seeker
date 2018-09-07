//! # Example
//! ```
//! let data = fs::read_to_string("search-index.js").unwrap();
//! let rustdoc: RustDoc = data.parse().unwrap();
//! let seeker = rustdoc.build().unwrap();
//! for i in seeker.search_regex("dedup.*") {
//!     println!("{}", i);
//! }
//! for i in seeker.search_edist("dedap", 1) {
//!     println!("{}", i);
//! }
//! ```

extern crate fst;
extern crate itertools;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate string_cache;

mod parser;
mod seeker;

pub use seeker::{DocItem, RustDoc, RustDocSeeker, TypeItem};
