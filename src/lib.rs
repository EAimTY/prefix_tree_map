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
    key: Option<Path<E, W>>,
    value: Option<V>,
    children: Option<Vec<Node<E, W, V>>>,
}

#[derive(Clone)]
pub struct TrieMapBuilder<E, W, V> {
    root: NodeBuilder<E, W, V>,
}

#[derive(Clone)]
struct NodeBuilder<E, W, V> {
    key: Option<Path<E, W>>,
    value: Option<V>,
    children: Option<BinaryHeap<NodeBuilder<E, W, V>>>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Path<E, W> {
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
                key: None,
                value: None,
                children: None,
            },
        }
    }

    pub fn insert(&mut self, key: impl IntoIterator<Item = Path<E, W>>, value: V) {
        unsafe {
            let mut node: *mut NodeBuilder<E, W, V> = &mut self.root;

            for part in key {
                if (*node).children.is_none() {
                    let mut children = BinaryHeap::new();
                    children.push(NodeBuilder::new(part));
                    (*node).children = Some(children);

                    let child = (*node).children.as_ref().unwrap().peek().unwrap();
                    let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                    node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                } else {
                    let children = (*node).children.as_mut().unwrap();

                    if let Some(child) = children
                        .iter()
                        .find(|node| node.key.as_ref() == Some(&part))
                    {
                        let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                        node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                    } else {
                        let part_cloned = part.clone();
                        children.push(NodeBuilder::new(part_cloned));

                        let child = children
                            .iter()
                            .find(|node| node.key.as_ref() == Some(&part))
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
        self.insert(key.into_iter().map(Path::Exact), value);
    }

    pub fn build(self) -> TrieMap<E, W, V> {
        TrieMap {
            root: Self::node_builder_to_node(self.root),
        }
    }

    fn node_builder_to_node(node_builder: NodeBuilder<E, W, V>) -> Node<E, W, V> {
        let key = node_builder.key;
        let value = node_builder.value;

        let children = node_builder.children.map(|children| {
            children
                .into_sorted_vec()
                .into_iter()
                .map(Self::node_builder_to_node)
                .collect()
        });

        Node {
            key,
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
    pub fn get(&self, key: &[E], param_map: &mut dyn Map<W, E>) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::new();
        let mut last_wildcard: Option<&Node<E, W, V>> = None;
        let mut current_part_idx = 0;

        let mut part_iter = key.iter();

        while let Some(part) = part_iter.next() {
            current_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = node.children.as_ref().unwrap();

                if children[0].key.as_ref().unwrap().is_wildcard() {
                    wildcards.push((current_part_idx, &children[0]));
                }

                if let Ok(idx) = children.binary_search_by(|node| {
                    let key = node.key.as_ref().unwrap();
                    key.as_ref().cmp(&Path::Exact(part))
                }) {
                    node = &children[idx];
                } else {
                    try_backtrack = true;
                }
            }

            if try_backtrack {
                if let Some((idx, wildcard_node)) = wildcards.pop() {
                    if let Some(last_wildcard) = last_wildcard {
                        let last_key = last_wildcard
                            .key
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unwrap_wildcard();
                        param_map.remove(last_key);
                    }

                    let wildcard_key = wildcard_node
                        .key
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap_wildcard();
                    let wildcard_value = &key[idx - 1];
                    param_map.insert(wildcard_key.to_owned(), wildcard_value.to_owned());

                    last_wildcard = Some(wildcard_node);

                    current_part_idx = idx;
                    part_iter = key[idx..].iter();
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

        for part in key {
            node.children.as_ref()?;

            let children = node.children.as_ref().unwrap();

            if let Ok(idx) = children.binary_search_by(|node| {
                let key = node.key.as_ref().unwrap();
                key.as_ref().cmp(&Path::Exact(part))
            }) {
                node = &children[idx];
            } else {
                return None;
            }
        }

        node.value.as_ref()
    }
}

impl<E, W> Path<E, W> {
    pub fn as_ref(&self) -> Path<&E, &W> {
        match self {
            Path::Exact(key) => Path::Exact(key),
            Path::Wildcard(key) => Path::Wildcard(key),
        }
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, Path::Wildcard(_))
    }

    pub fn is_exact(&self) -> bool {
        matches!(self, Path::Exact(_))
    }

    fn unwrap_wildcard(self) -> W {
        if let Path::Wildcard(key) = self {
            key
        } else {
            panic!("Wrong path type");
        }
    }
}

impl<E, W> PartialOrd for Path<E, W>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &Path<E, W>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E, W> Ord for Path<E, W>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Path::Exact(key), Path::Exact(other_key)) => key.cmp(other_key),
            (Path::Wildcard(key), Path::Wildcard(other_key)) => key.cmp(other_key),
            (Path::Exact(_), Path::Wildcard(_)) => Ordering::Greater,
            (Path::Wildcard(_), Path::Exact(_)) => Ordering::Less,
        }
    }
}

impl<E, W, V> PartialEq for Node<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
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
        self.key.partial_cmp(&other.key)
    }
}

impl<E, W, V> Ord for Node<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
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
    fn new(key: Path<E, W>) -> Self {
        Self {
            key: Some(key),
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
        self.key == other.key
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
        self.key.partial_cmp(&other.key)
    }
}

impl<E, W, V> Ord for NodeBuilder<E, W, V>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

pub trait Map<W, E> {
    fn insert(&mut self, key: W, value: E);
    fn remove(&mut self, key: &W);
}

impl<W, E> Map<W, E> for BTreeMap<W, E>
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
