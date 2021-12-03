use trie_map::TrieMap;

fn main() {
    let mut trie = TrieMap::new();

    trie.insert_exact(["hello"], "world");
    trie.insert_exact(["hello", "world"], "again");

    println!("{}", trie.get_exact(&["hello"]).unwrap());
    println!("{}", trie.get_exact(&["hello", "world"]).unwrap());
}
