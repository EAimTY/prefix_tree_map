# prefix_tree_map
A Rust implementation of generic prefix tree (trie) map with wildcard capture support.

[![Version](https://img.shields.io/crates/v/prefix_tree_map.svg?style=flat)](https://crates.io/crates/prefix_tree_map)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/prefix_tree_map)
[![License](https://img.shields.io/crates/l/prefix_tree_map.svg?style=flat)](https://github.com/EAimTY/prefix_tree_map/blob/master/LICENSE)

## Design
[Trie](https://en.wikipedia.org/wiki/Trie) is a good data structure for storing key-value pairs with wildcard support ability.

This prefix tree map implementation supports any type of key and value, as long as key parts are implemented the `Ord` and `Clone` trait. Internally, nodes are stored in a sorted `Vec`. So technically it can achieve `O(log n)` time complexity on finding every node by using binary search on the sorted `Vec`.

Using as a route-table-like structure could be the best scenario for this crate.

## Pros and Cons

### Pros
- **Fast and efficient**
- **Wildcard Capture Support** - Capture wildcard characters in a map while matching.
- **Generalization** - Supports any type of key and value, as long as key parts are implemented the `Ord` and `Clone` trait.
- **Capture Map Customization** - Customize the way key parts captured by wildcard stored.
- **No recursion in find operations** - Rather than store the whole context of every node searching, this prefix tree map only store those tiny wildcard node pointers for backtracking on heap.

### Cons
- The map itself is immutable, because the map builder is using Binary Heap to sort the nodes when they are inserted. We can't get a item from a Binary Heap without iterating the whole Binary Heap. Once the `build()` is called, Binary Heaps are converted into sorted `Vec`s. We can't push any item to the `Vec` without a whole sort operation.
- Currently, a single wildcard cannot be matched more than one time. It means `word` can be matched by `w**d`, not `w*d`.

## Usage
```rust
use prefix_tree_map::PrefixTreeMapBuilder;

let mut map_builder = PrefixTreeMapBuilder::new();

// To insert an exact key path, call `insert_exact()`
map_builder.insert_exact(["path", "to", "value"], "value0");

// Insert into a existed key path could overwrite the value in it
map_builder.insert_exact(["path", "to", "value"], "value1");

// To insert an key path with wildcards, mark key parts using `prefix_tree_map::KeyPart` and call `insert()`
use prefix_tree_map::KeyPart;

map_builder.insert(
    [
        KeyPart::Exact("path"),
        KeyPart::Wildcard("to"),
        KeyPart::Exact("value"),
    ],
    "value2",
);

// Anything implemented trait `FromIterator` can be inserted as a key path:
let path = "path/to/anothor/value";
map_builder.insert_exact(path.split('/'), "value3");

let anothor_path = "path/to/:some/value";
map_builder.insert(
    anothor_path.split('/').map(|part| {
        if part.starts_with(':') {
            KeyPart::Wildcard(part)
        } else {
            KeyPart::Exact(part)
        }
    }),
    "value4",
);

// Then build the map
let map = map_builder.build();

// Find a value without matching any wildcard part
assert_eq!(
    Some(&"value3"),
    map.find_exact(&["path", "to", "anothor", "value"])
);

// Find a value with matching wildcard part
assert_eq!(Some(&"value4"), map.find(&["path", "to", "a", "value"]));

// `KeyPart::Exact` has a higher match priority than `KeyPart::Wildcard`
assert_eq!(Some(&"value3"), map.find(&["path", "to", "anothor", "value"]));

// Find a value with matching wildcard part, and store captured matched wildcard parts in a map
use std::collections::HashMap;

let mut captures = HashMap::new();
assert_eq!(
    Some(&"value4"),
    map.find_and_capture(&["path", "to", "a", "value"], &mut captures)
);

assert_eq!(Some(&"a"), captures.get(&":some"));
```

Customizing Capture map:
```rust
struct Map {
    pub data: [Option<String>; 2],
}

impl Map {
    fn new() -> Self {
        Self { data: [None, None] }
    }
}

use prefix_tree_map::Captures;

impl Captures<&str, &str> for Map {
    fn insert(&mut self, key: &str, value: &str) {
        match key {
            ":user_id" => self.data[0] = Some(value.to_string()),
            ":product_id" => self.data[1] = Some(value.to_string()),
            _ => (),
        }
    }
}

fn capture() {
    use prefix_tree_map::{KeyPart, PrefixTreeMapBuilder};

    let mut builder = PrefixTreeMapBuilder::new();

    builder.insert(
        [
            KeyPart::Exact("user"),
            KeyPart::Wildcard(":user_id"),
            KeyPart::Exact("home"),
        ],
        "user",
    );

    builder.insert(
        [
            KeyPart::Exact("product"),
            KeyPart::Wildcard(":product_id"),
            KeyPart::Exact("info"),
        ],
        "product",
    );

    let map = builder.build();
    let mut captures = Map::new();

    map.find_and_capture(
        &"/user/00000/home".split('/').collect::<Vec<_>>(),
        &mut captures,
    );

    assert_eq!("00000", captures.data[0].as_ref().unwrap());
}
```

For more infomation, check out [examples/router.rs](https://github.com/EAimTY/prefix_tree_map/blob/master/examples/router.rs)

## Examples

Check [examples](https://github.com/EAimTY/prefix_tree_map/tree/master/examples).

## License
GNU General Public License v3.0
