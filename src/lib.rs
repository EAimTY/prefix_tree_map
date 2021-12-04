use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
};

#[derive(Clone)]
pub struct TrieMap<P, V> {
    root: Node<P, V>,
}

#[derive(Clone)]
pub struct TrieMapBuilder<P, V> {
    root: NodeBuilder<P, V>,
}

#[derive(Clone)]
struct Node<P, V> {
    key: Option<Path<P>>,
    value: Option<V>,
    children: Option<Vec<Node<P, V>>>,
}

#[derive(Clone)]
struct NodeBuilder<P, V> {
    key: Option<Path<P>>,
    value: Option<V>,
    children: Option<BinaryHeap<NodeBuilder<P, V>>>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Path<P> {
    Exact(P),
    Wildcard(P),
}

impl<P, V> TrieMapBuilder<P, V>
where
    P: Clone + Ord + PartialEq,
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

    pub fn insert(&mut self, key: impl IntoIterator<Item = Path<P>>, value: V) {
        unsafe {
            let mut node: *mut NodeBuilder<P, V> = &mut self.root;

            for part in key {
                if (*node).children.is_none() {
                    let mut children = BinaryHeap::new();
                    children.push(NodeBuilder::new(part));
                    (*node).children = Some(children);

                    let child = (*node).children.as_ref().unwrap().peek().unwrap();
                    let child_const_ptr = child as *const NodeBuilder<P, V>;
                    node = child_const_ptr as *mut NodeBuilder<P, V>;
                } else {
                    let children = (*node).children.as_mut().unwrap();

                    if let Some(child) = children
                        .iter()
                        .find(|node| node.key.as_ref() == Some(&part))
                    {
                        let child_const_ptr = child as *const NodeBuilder<P, V>;
                        node = child_const_ptr as *mut NodeBuilder<P, V>;
                    } else {
                        let part_cloned = part.clone();
                        children.push(NodeBuilder::new(part_cloned));

                        let child = children
                            .iter()
                            .find(|node| node.key.as_ref() == Some(&part))
                            .unwrap();
                        let child_const_ptr = child as *const NodeBuilder<P, V>;
                        node = child_const_ptr as *mut NodeBuilder<P, V>;
                    }
                }
            }

            (*node).value = Some(value);
        }
    }

    pub fn insert_exact(&mut self, key: impl IntoIterator<Item = P>, value: V) {
        self.insert(key.into_iter().map(Path::Exact), value);
    }

    pub fn build(self) -> TrieMap<P, V> {
        TrieMap {
            root: Self::node_builder_to_node(self.root),
        }
    }

    fn node_builder_to_node(node_builder: NodeBuilder<P, V>) -> Node<P, V> {
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

impl<P, V> TrieMap<P, V>
where
    P: Clone + Ord + PartialEq,
{
    pub fn get(&self, key: &[P], param_map: &mut BTreeMap<P, P>) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::new();
        let mut last_wildcard: Option<&Node<P, V>> = None;
        let mut current_part_idx = 0;

        let mut part_iter = key.iter();

        while let Some(part) = part_iter.next() {
            current_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = node.children.as_ref().unwrap();

                if let Some(wildcard) = children.iter().find(|node| {
                    let key = node.key.as_ref().unwrap();
                    key.is_wildcard()
                }) {
                    wildcards.push((current_part_idx, wildcard));
                }

                if let Some(child) = children.iter().find(|node| {
                    let key = node.key.as_ref().unwrap();
                    key.as_ref() == Path::Exact(part)
                }) {
                    node = child;
                } else {
                    try_backtrack = true;
                }
            }

            if try_backtrack {
                if let Some((idx, wildcard_node)) = wildcards.pop() {
                    if let Some(last_wildcard) = last_wildcard {
                        let last_key = last_wildcard.key.as_ref().unwrap().as_ref().unwrap();
                        param_map.remove(last_key);
                    }

                    let wildcard_key = wildcard_node.key.as_ref().unwrap().as_ref().unwrap();
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

    pub fn get_exact(&self, key: &[P]) -> Option<&V> {
        let mut node = &self.root;

        for part in key {
            node.children.as_ref()?;

            let children = node.children.as_ref().unwrap();

            if let Some(child) = children
                .iter()
                .find(|node| node.key.as_ref().map(|key| key.as_ref()) == Some(Path::Exact(part)))
            {
                node = child;
            } else {
                return None;
            }
        }

        node.value.as_ref()
    }
}

impl<P> Path<P> {
    pub fn as_ref(&self) -> Path<&P> {
        match self {
            Path::Exact(key) => Path::Exact(key),
            Path::Wildcard(key) => Path::Wildcard(key),
        }
    }

    pub fn unwrap(self) -> P {
        match self {
            Path::Exact(key) => key,
            Path::Wildcard(key) => key,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, Path::Wildcard(_))
    }

    pub fn is_exact(&self) -> bool {
        matches!(self, Path::Exact(_))
    }
}

impl<P> PartialOrd for Path<P>
where
    P: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &Path<P>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P> Ord for Path<P>
where
    P: Clone + Ord + PartialEq,
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

impl<P, V> Default for TrieMapBuilder<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P, V> NodeBuilder<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn new(key: Path<P>) -> Self {
        Self {
            key: Some(key),
            value: None,
            children: None,
        }
    }
}

impl<P, V> PartialEq for Node<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<P, V> Eq for Node<P, V> where P: Clone + Ord + PartialEq {}

impl<P, V> PartialOrd for Node<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<P, V> Ord for Node<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<P, V> PartialEq for NodeBuilder<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<P, V> Eq for NodeBuilder<P, V> where P: Clone + Ord + PartialEq {}

impl<P, V> PartialOrd for NodeBuilder<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<P, V> Ord for NodeBuilder<P, V>
where
    P: Clone + Ord + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
