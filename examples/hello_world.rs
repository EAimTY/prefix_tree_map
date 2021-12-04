use std::collections::BTreeMap;
use trie_map::{Path, TrieMapBuilder};

fn main() {
    let mut trie = TrieMapBuilder::new();

    trie.insert_exact([1, 2, 3], "world");
    trie.insert_exact([7, 5, 2, 3], "world");
    trie.insert_exact([2, 5], "world");
    trie.insert_exact([4, 2, 3, 1, 9], "world");
    trie.insert(
        [
            Path::Exact(1),
            Path::Wildcard(2),
            Path::Exact(3),
            Path::Exact(4),
        ],
        "again",
    );

    let trie = trie.build();

    println!("{}", trie.get_exact(&[1, 2, 3]).unwrap());

    let mut map = BTreeMap::new();
    println!("{}", trie.get(&[1, 22, 3, 4], &mut map).unwrap());
    println!("{:?}", map);
}
