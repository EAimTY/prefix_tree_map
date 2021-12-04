use crate::{
    key_part::KeyPart,
    trie_map::{Node, TrieMap},
};
use std::collections::BinaryHeap;

#[derive(Clone)]
pub struct TrieMapBuilder<E, W, V> {
    root: NodeBuilder<E, W, V>,
}

#[derive(Clone)]
struct NodeBuilder<E, W, V> {
    key_part: Option<KeyPart<E, W>>,
    value: Option<V>,
    children: Option<BinaryHeap<NodeBuilder<E, W, V>>>,
}

impl<E, W, V> TrieMapBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    pub fn new() -> Self {
        Self {
            root: NodeBuilder {
                key_part: None,
                value: None,
                children: None,
            },
        }
    }

    pub fn insert(&mut self, key: impl IntoIterator<Item = KeyPart<E, W>>, value: V) {
        unsafe {
            let mut node: *mut NodeBuilder<E, W, V> = &mut self.root;

            for key_part in key {
                if (*node).children.is_none() {
                    let mut children = BinaryHeap::new();
                    children.push(NodeBuilder::new(key_part));
                    (*node).children = Some(children);

                    let child = (*node).children.as_ref().unwrap().peek().unwrap();
                    let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                    node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                } else {
                    let children = (*node).children.as_mut().unwrap();

                    if let Some(child) = children
                        .iter()
                        .find(|child| child.key_part.as_ref() == Some(&key_part))
                    {
                        let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                        node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                    } else {
                        let key_part_cloned = key_part.clone();
                        children.push(NodeBuilder::new(key_part_cloned));

                        let child = children
                            .iter()
                            .find(|child| child.key_part.as_ref() == Some(&key_part))
                            .unwrap();
                        let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                        node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                    }
                }
            }

            (*node).value = Some(value);
        }
    }

    pub fn insert_exact(&mut self, key: impl IntoIterator<Item = E>, value: V) {
        self.insert(key.into_iter().map(KeyPart::Exact), value);
    }

    pub fn build(self) -> TrieMap<E, W, V> {
        TrieMap {
            root: Self::node_builder_to_node(self.root),
        }
    }

    fn node_builder_to_node(node_builder: NodeBuilder<E, W, V>) -> Node<E, W, V> {
        let key_part = node_builder.key_part;
        let value = node_builder.value;

        let children = node_builder.children.map(|children| {
            children
                .into_sorted_vec()
                .into_iter()
                .map(Self::node_builder_to_node)
                .collect()
        });

        Node {
            key_part,
            value,
            children,
        }
    }
}

impl<E, W, V> Default for TrieMapBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E, W, V> NodeBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn new(key_part: KeyPart<E, W>) -> Self {
        Self {
            key_part: Some(key_part),
            value: None,
            children: None,
        }
    }
}

impl<E, W, V> PartialEq for NodeBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_part == other.key_part
    }
}

impl<E, W, V> Eq for NodeBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
}

impl<E, W, V> PartialOrd for NodeBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key_part.partial_cmp(&other.key_part)
    }
}

impl<E, W, V> Ord for NodeBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key_part.cmp(&other.key_part)
    }
}
