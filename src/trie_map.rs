use crate::{capture_map::CaptureMap, key_part::KeyPart};
use std::fmt::Debug;

/// The trie map.
#[derive(Clone)]
pub struct TrieMap<E, W, V> {
    pub(crate) root: Node<E, W, V>,
}

#[derive(Clone)]
pub(crate) struct Node<E, W, V> {
    pub(crate) key_part: Option<KeyPart<E, W>>,
    pub(crate) value: Option<V>,
    pub(crate) children: Option<Vec<Node<E, W, V>>>,
}

impl<E, W, V> TrieMap<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    /// Find a value with matching wildcard part.
    ///
    /// Return the smallest value that matches the given key.
    pub fn find(&self, key: &[E]) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::new();

        let mut key_part_iter = key.iter();
        let mut key_part_idx = 0;

        while let Some(key_part) = key_part_iter.next() {
            key_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = node.children.as_ref().unwrap();

                children
                    .iter()
                    .take_while(|child| child.key_part.as_ref().unwrap().is_wildcard())
                    .for_each(|child| {
                        wildcards.push((key_part_idx, child));
                    });

                if let Ok(child_idx) = children.binary_search_by(|child| {
                    let child_key_part = child.key_part.as_ref().unwrap();
                    child_key_part.as_ref().cmp(&KeyPart::Exact(key_part))
                }) {
                    node = &children[child_idx];
                } else {
                    try_backtrack = true;
                }
            }

            if key_part_idx == key.len() && node.value.is_none() {
                try_backtrack = true;
            }

            if try_backtrack {
                if let Some((wildcard_key_part_idx, wildcard_node)) = wildcards.pop() {
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

    /// Find a value with matching wildcard part, and store captured matched wildcard parts in a map.
    ///
    /// Return the smallest value that matches the given key.
    pub fn find_and_capture<M: CaptureMap<W, E>>(&self, key: &[E], captures: &mut M) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::new();
        let mut last_wildcard_node: Option<&Node<E, W, V>> = None;

        let mut key_part_iter = key.iter();
        let mut key_part_idx = 0;

        captures.clear();

        while let Some(key_part) = key_part_iter.next() {
            key_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = node.children.as_ref().unwrap();

                children
                    .iter()
                    .take_while(|child| child.key_part.as_ref().unwrap().is_wildcard())
                    .for_each(|child| {
                        wildcards.push((key_part_idx, child));
                    });

                if let Ok(child_idx) = children.binary_search_by(|child| {
                    let child_key_part = child.key_part.as_ref().unwrap();
                    child_key_part.as_ref().cmp(&KeyPart::Exact(key_part))
                }) {
                    node = &children[child_idx];
                } else {
                    try_backtrack = true;
                }
            }

            if key_part_idx == key.len() && node.value.is_none() {
                try_backtrack = true;
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
                        captures.remove(last_wildcard_key_part);
                    }

                    let wildcard_key_part = wildcard_node
                        .key_part
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap_wildcard();
                    let matched_key_part = &key[wildcard_key_part_idx - 1];
                    captures.insert(wildcard_key_part.to_owned(), matched_key_part.to_owned());

                    last_wildcard_node = Some(wildcard_node);

                    key_part_idx = wildcard_key_part_idx;
                    key_part_iter = key[wildcard_key_part_idx..].iter();
                    node = wildcard_node;
                } else {
                    return None;
                }
            }
        }

        if node.value.is_none() {
            captures.clear();
        }

        node.value.as_ref()
    }

    /// Find a value without matching wildcard part.
    pub fn find_exact(&self, key: &[E]) -> Option<&V> {
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

impl<E: Debug, W: Debug, V: Debug> Debug for TrieMap<E, W, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TrieMap {{ root: {:?} }}", self.root)
    }
}

impl<E: Debug, W: Debug, V: Debug> Debug for Node<E, W, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ key_part: {:?}, value: {:?}, children: {:?} }}",
            self.key_part, self.value, self.children
        )
    }
}

impl<E, W, V> PartialEq for Node<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn eq(&self, other: &Self) -> bool {
        self.key_part == other.key_part
    }
}

impl<E, W, V> Eq for Node<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
}

impl<E, W, V> PartialOrd for Node<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key_part.partial_cmp(&other.key_part)
    }
}

impl<E, W, V> Ord for Node<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key_part.cmp(&other.key_part)
    }
}
