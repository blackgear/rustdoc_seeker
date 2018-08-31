extern crate rustdoc_seeker;
use rustdoc_seeker::RustDoc;
use std::fs;

fn main() {
    let data = fs::read_to_string("search-index.js").unwrap();
    let rustdoc: RustDoc = data.parse().unwrap();
    let seeker = rustdoc.build().unwrap();
    println!("Regex");
    for i in seeker.search_regex("dedup.*") {
        println!("{}", i);
    }
    println!("\nEdit Distance");
    for i in seeker.search_edist("dedap", 1) {
        println!("{}", i);
    }
}
