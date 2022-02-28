use crate::{
    key_part::KeyPart,
    prefix_tree_map::{Node, PrefixTreeMap},
    std_lib::{BinaryHeap, Ordering},
};

/// The prefix tree map builder
#[derive(Clone)]
pub struct PrefixTreeMapBuilder<E, W, V> {
    root: NodeBuilder<E, W, V>,
}

#[derive(Clone)]
struct NodeBuilder<E, W, V> {
    key_part: Option<KeyPart<E, W>>,
    value: Option<V>,
    children: Option<BinaryHeap<NodeBuilder<E, W, V>>>,
}

impl<E, W, V> PrefixTreeMapBuilder<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    /// Create a new `PrefixTreeMapBuilder`
    pub fn new() -> Self {
        Self {
            root: NodeBuilder {
                key_part: None,
                value: None,
                children: None,
            },
        }
    }

    /// Insert a new value into the prefix tree map
    ///
    /// Key parts need to be marked by [`KeyPart`](enum.KeyPart.html)
    ///
    /// Insert into a existed key path could overwrite the value in it
    pub fn insert(&mut self, key: impl IntoIterator<Item = KeyPart<E, W>>, value: V) {
        let mut node = &mut self.root as *mut NodeBuilder<E, W, V>;

        for key_part in key {
            if unsafe { (*node).children.is_none() } {
                let mut children = BinaryHeap::new();
                children.push(NodeBuilder::new(key_part));

                unsafe {
                    (*node).children = Some(children);
                }

                let child = unsafe {
                    (*node)
                        .children
                        .as_ref()
                        .unwrap_unchecked()
                        .peek()
                        .unwrap_unchecked()
                };

                let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                node = child_const_ptr as *mut NodeBuilder<E, W, V>;
            } else {
                let children = unsafe { (*node).children.as_mut().unwrap_unchecked() };

                if let Some(child) = children
                    .iter()
                    .find(|child| child.key_part.as_ref() == Some(&key_part))
                {
                    let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                    node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                } else {
                    let key_part_cloned = key_part.clone();
                    children.push(NodeBuilder::new(key_part_cloned));

                    let child = unsafe {
                        children
                            .iter()
                            .find(|child| child.key_part.as_ref() == Some(&key_part))
                            .unwrap_unchecked()
                    };

                    let child_const_ptr = child as *const NodeBuilder<E, W, V>;
                    node = child_const_ptr as *mut NodeBuilder<E, W, V>;
                }
            }
        }

        unsafe {
            (*node).value = Some(value);
        }
    }

    /// Insert a new value in an exact key path
    pub fn insert_exact(&mut self, key: impl IntoIterator<Item = E>, value: V) {
        self.insert(key.into_iter().map(KeyPart::Exact), value);
    }

    /// Build the prefix tree map
    pub fn build(self) -> PrefixTreeMap<E, W, V> {
        PrefixTreeMap {
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

impl<E, W, V> Default for PrefixTreeMapBuilder<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E, W, V> NodeBuilder<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
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
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_part == other.key_part
    }
}

impl<E, W, V> Eq for NodeBuilder<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
}

impl<E, W, V> PartialOrd for NodeBuilder<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key_part.partial_cmp(&other.key_part)
    }
}

impl<E, W, V> Ord for NodeBuilder<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.key_part.cmp(&other.key_part)
    }
}
