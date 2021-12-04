#[derive(Debug, Clone)]
pub struct TrieMap<P, V> {
    root: Node<P, V>,
}

#[derive(Debug, Clone)]
struct Node<P, V> {
    key: Option<Path<P>>,
    value: Option<V>,
    children: Option<Vec<Node<P, V>>>,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum Path<P> {
    Exact(P),
    Wildcard(P),
}

impl<P, V> TrieMap<P, V>
where
    P: Ord + PartialEq + ToOwned<Owned = P>,
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

    pub fn insert(&mut self, key: impl IntoIterator<Item = Path<P>>, value: V) {
        unsafe {
            let mut node: *mut Node<P, V> = &mut self.root;

            for part in key {
                if (*node).children.is_none() {
                    let children = vec![Node::new(part)];
                    (*node).children = Some(children);

                    node = &mut (*node).children.as_mut().unwrap()[0];
                } else {
                    let children = (*node).children.as_mut().unwrap();

                    if let Some(child) = children
                        .iter_mut()
                        .find(|node| node.key.as_ref() == Some(&part))
                    {
                        node = child;
                    } else {
                        children.push(Node::new(part));
                        let child_idx = children.len() - 1;
                        node = &mut children[child_idx];
                    }
                }
            }

            (*node).value = Some(value);
        }
    }

    pub fn insert_exact(&mut self, key: impl IntoIterator<Item = P>, value: V) {
        self.insert(key.into_iter().map(Path::Exact), value);
    }

    pub fn get(&self, key: &[P]) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::new();
        let mut current_part_idx = 0usize;

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
                    wildcards.push((current_part_idx + 1, wildcard));
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

impl<P, V> Default for TrieMap<P, V>
where
    P: Ord + PartialEq + ToOwned<Owned = P>,
{
    fn default() -> Self {
        Self::new()
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

impl<P, V> Node<P, V>
where
    P: Ord + PartialEq + ToOwned<Owned = P>,
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
    P: Ord + PartialEq + ToOwned<Owned = P>,
{
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<P, V> Eq for Node<P, V> where P: Ord + PartialEq + ToOwned<Owned = P> {}

impl<P, V> PartialOrd for Node<P, V>
where
    P: Ord + PartialEq + ToOwned<Owned = P>,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<P, V> Ord for Node<P, V>
where
    P: Ord + PartialEq + ToOwned<Owned = P>,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
