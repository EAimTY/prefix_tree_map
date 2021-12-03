use std::collections::BTreeMap;

use trie_map::{Path, TrieMap};

fn main() {
    let mut trie = TrieMap::new();

    trie.insert_exact(["hello"], "world");
    trie.insert([Path::Exact("hello"), Path::Wildcard("something")], "again");

    println!("{}", trie.get_exact(&["hello"]).unwrap());

    let mut map = BTreeMap::new();
    println!("{}", trie.get(&["hello", "world"], &mut map).unwrap());
    println!("{:?}", map);
}
