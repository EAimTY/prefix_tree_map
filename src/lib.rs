#[derive(Clone)]
pub struct TrieMap<K, V> {
    root: Node<K, V>,
}

#[derive(Clone)]
struct Node<K, V> {
    key: Option<Key<K>>,
    value: Option<V>,
    children: Option<Vec<Node<K, V>>>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Key<K> {
    Exact(K),
    Wildcard(K),
}

impl<K, V> TrieMap<K, V>
where
    K: PartialEq + Ord,
{
    pub fn new() -> Self {
        Self {
            root: Node {
                key: None,
                value: None,
                children: None,
            },
        }
    }

    pub fn insert(&mut self, key: impl IntoIterator<Item = Key<K>>, value: V) {
        unsafe {
            let mut node: *mut Node<K, V> = &mut self.root;

            for part in key {
                if (*node).children.is_none() {
                    let children = vec![Node::new(part, None)];
                    (*node).children = Some(children);

                    node = &mut (*node).children.as_mut().unwrap()[0];
                } else {
                    let children = (*node).children.as_mut().unwrap();

                    if let Some(same_node) = children
                        .iter_mut()
                        .find(|node| node.key.as_ref() == Some(&part))
                    {
                        node = same_node;
                    }
                }
            }

            (*node).value = Some(value);
        }
    }

    pub fn insert_exact(&mut self, key: impl IntoIterator<Item = K>, value: V) {
        self.insert(key.into_iter().map(Key::Exact), value);
    }

    pub fn get_exact(&self, key: &[K]) -> Option<&V> {
        unsafe {
            let mut node: *const Node<K, V> = &self.root;

            for part in key {
                (*node).children.as_ref()?;

                let children = (*node).children.as_ref().unwrap();

                if let Some(same_node) = children.iter().find(|node| {
                    node.key.as_ref().map(|key| key.as_ref()) == Some(Key::Exact(part))
                }) {
                    node = same_node;
                } else {
                    return None;
                }
            }

            (*node).value.as_ref()
        }
    }
}

impl<K, V> Default for TrieMap<K, V>
where
    K: PartialEq + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Key<K> {
    fn as_ref(&self) -> Key<&K> {
        match self {
            Key::Exact(key) => Key::Exact(key),
            Key::Wildcard(key) => Key::Wildcard(key),
        }
    }
}

impl<K, V> Node<K, V>
where
    K: PartialEq + Ord,
{
    fn new(key: Key<K>, value: Option<V>) -> Self {
        Self {
            key: Some(key),
            value,
            children: None,
        }
    }
}

impl<K, V> PartialEq for Node<K, V>
where
    K: PartialEq + Ord,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<K, V> Eq for Node<K, V> where K: PartialEq + Ord {}

impl<K, V> PartialOrd for Node<K, V>
where
    K: PartialEq + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<K, V> Ord for Node<K, V>
where
    K: PartialEq + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
