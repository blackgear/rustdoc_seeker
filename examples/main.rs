extern crate fst;
extern crate fst_levenshtein;
extern crate fst_regex;
extern crate rustdoc_seeker;
use fst::Automaton;
use rustdoc_seeker::RustDoc;
use std::fs;

fn main() {
    let data = fs::read_to_string("search-index.js").unwrap();
    let rustdoc: RustDoc = data.parse().unwrap();
    let seeker = rustdoc.build().unwrap();

    let regex = fst_regex::Regex::new(".*dedup.*").unwrap();
    for i in seeker.search(&regex) {
        println!("Regex {}", i);
    }

    let edist = fst_levenshtein::Levenshtein::new("dedXp", 1).unwrap();
    for i in seeker.search(&edist) {
        println!("Edit Distance {}", i);
    }

    let subsq = fst::automaton::Subsequence::new("dedup");
    for i in seeker.search(&subsq) {
        println!("Subsequence {}", i);
    }

    let union = subsq.union(regex);
    for i in seeker.search(&union) {
        println!("Union {}", i);
    }

    let starts = edist.starts_with();
    for i in seeker.search(&starts) {
        println!("Starts_with {}", i);
    }
}
