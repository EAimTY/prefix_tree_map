use std::collections::BTreeMap;

use trie_map::{Path, TrieMap};

fn main() {
    let mut trie = TrieMap::new();

    trie.insert_exact([1, 2, 3], "world");
    trie.insert(
        [
            Path::Exact(1),
            Path::Wildcard(2),
            Path::Exact(3),
            Path::Exact(4),
        ],
        "again",
    );

    println!("{:#?}", trie);
    // println!("{}", trie.get_exact(&[1, 2, 3]).unwrap());

    // let mut map = BTreeMap::new();
    println!("{}", trie.get(&[1, 2, 3, 4]).unwrap());
    // println!("{:?}", map);
}
