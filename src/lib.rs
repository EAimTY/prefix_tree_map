use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
};

#[derive(Clone)]
pub struct TrieMap<E, W, V> {
    root: Node<E, W, V>,
}

#[derive(Clone)]
struct Node<E, W, V> {
    key_part: Option<KeyPart<E, W>>,
    value: Option<V>,
    children: Option<Vec<Node<E, W, V>>>,
}

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

#[derive(Clone, Eq, PartialEq)]
pub enum KeyPart<E, W> {
    Exact(E),
    Wildcard(W),
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

impl<E, W, V> TrieMap<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    pub fn get<M: CaptureMap<W, E>>(&self, key: &[E], param_map: &mut M) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::new();
        let mut last_wildcard_node: Option<&Node<E, W, V>> = None;

        let mut key_part_iter = key.iter();
        let mut key_part_idx = 0;

        while let Some(key_part) = key_part_iter.next() {
            key_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = node.children.as_ref().unwrap();

                if children[0].key_part.as_ref().unwrap().is_wildcard() {
                    wildcards.push((key_part_idx, &children[0]));
                }

                if let Ok(child_idx) = children.binary_search_by(|child| {
                    let child_key_part = child.key_part.as_ref().unwrap();
                    child_key_part.as_ref().cmp(&KeyPart::Exact(key_part))
                }) {
                    node = &children[child_idx];
                } else {
                    try_backtrack = true;
                }
            }

            if try_backtrack {
                if let Some((wildcard_key_part_idx, wildcard_node)) = wildcards.pop() {
                    if let Some(last_wildcard_node) = last_wildcard_node {
                        let last_wildcard_key_part = last_wildcard_node
                            .key_part
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap_wildcard();
                        param_map.remove(last_wildcard_key_part);
                    }

                    let wildcard_key_part = wildcard_node
                        .key_part
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap_wildcard();
                    let matched_key_part = &key[wildcard_key_part_idx - 1];
                    param_map.insert(wildcard_key_part.to_owned(), matched_key_part.to_owned());

                    last_wildcard_node = Some(wildcard_node);

                    key_part_idx = wildcard_key_part_idx;
                    key_part_iter = key[wildcard_key_part_idx..].iter();
                    node = wildcard_node;
                } else {
                    return None;
                }
            }
        }

        node.value.as_ref()
    }

    pub fn get_exact(&self, key: &[E]) -> Option<&V> {
        let mut node = &self.root;

        for key_part in key {
            node.children.as_ref()?;

            let children = node.children.as_ref().unwrap();

            if let Ok(child_idx) = children.binary_search_by(|child| {
                let child_key_part = child.key_part.as_ref().unwrap();
                child_key_part.as_ref().cmp(&KeyPart::Exact(key_part))
            }) {
                node = &children[child_idx];
            } else {
                return None;
            }
        }

        node.value.as_ref()
    }
}

impl<E, W> KeyPart<E, W> {
    pub fn as_ref(&self) -> KeyPart<&E, &W> {
        match self {
            KeyPart::Exact(key) => KeyPart::Exact(key),
            KeyPart::Wildcard(key) => KeyPart::Wildcard(key),
        }
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, KeyPart::Wildcard(_))
    }

    pub fn is_exact(&self) -> bool {
        matches!(self, KeyPart::Exact(_))
    }

    fn unwrap_wildcard(self) -> W {
        if let KeyPart::Wildcard(key) = self {
            key
        } else {
            panic!();
        }
    }
}

impl<E, W> PartialOrd for KeyPart<E, W>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &KeyPart<E, W>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E, W> Ord for KeyPart<E, W>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (KeyPart::Exact(key_part), KeyPart::Exact(other_key_part)) => {
                key_part.cmp(other_key_part)
            }
            (KeyPart::Wildcard(key_part), KeyPart::Wildcard(other_key_part)) => {
                key_part.cmp(other_key_part)
            }
            (KeyPart::Exact(_), KeyPart::Wildcard(_)) => Ordering::Greater,
            (KeyPart::Wildcard(_), KeyPart::Exact(_)) => Ordering::Less,
        }
    }
}

impl<E, W, V> PartialEq for Node<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_part == other.key_part
    }
}

impl<E, W, V> Eq for Node<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
}

impl<E, W, V> PartialOrd for Node<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key_part.partial_cmp(&other.key_part)
    }
}

impl<E, W, V> Ord for Node<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key_part.cmp(&other.key_part)
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

pub trait CaptureMap<W, E> {
    fn insert(&mut self, key: W, value: E);
    fn remove(&mut self, key: &W);
}

impl<W, E> CaptureMap<W, E> for BTreeMap<W, E>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }

    fn remove(&mut self, key: &W) {
        self.remove(key);
    }
}
