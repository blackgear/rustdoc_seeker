extern crate fst;
extern crate regex_automata;
extern crate rustdoc_seeker;
use fst::automaton::{Levenshtein, Subsequence};
use fst::Automaton;
use regex_automata::DenseDFA;
use rustdoc_seeker::RustDoc;
use std::fs;

fn main() {
    let data = fs::read_to_string("search-index.js").unwrap();
    let rustdoc: RustDoc = data.parse().unwrap();
    let seeker = rustdoc.build();

    let dfa = DenseDFA::new(".*dedup.*").unwrap();
    for i in seeker.search(&dfa) {
        println!("Regex {}", i);
    }

    let edist = Levenshtein::new("dedXp", 1).unwrap();
    for i in seeker.search(&edist) {
        println!("Edit Distance {}", i);
    }

    let subsq = Subsequence::new("dedup");
    for i in seeker.search(&subsq) {
        println!("Subsequence {}", i);
    }

    let union = subsq.union(dfa);
    for i in seeker.search(&union) {
        println!("Union {}", i);
    }

    let starts = edist.starts_with();
    for i in seeker.search(&starts) {
        println!("Starts_with {}", i);
    }
}
