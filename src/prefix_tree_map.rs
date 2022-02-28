use crate::{
    captures::Captures,
    key_part::KeyPart,
    std_lib::{Debug, FmtResult, Formatter, Ordering, Vec},
};

/// The prefix tree map
#[derive(Clone)]
pub struct PrefixTreeMap<E, W, V> {
    pub(crate) root: Node<E, W, V>,
    pub(crate) max_wildcard_depth: usize,
}

#[derive(Clone)]
pub(crate) struct Node<E, W, V> {
    pub(crate) key_part: Option<KeyPart<E, W>>,
    pub(crate) value: Option<V>,
    pub(crate) children: Option<Vec<Node<E, W, V>>>,
}

impl<E, W, V> PrefixTreeMap<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    /// Find a value with matching wildcard part
    ///
    /// Return the smallest value that matches the given key
    pub fn find(&self, key: &[E]) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::with_capacity(self.max_wildcard_depth);

        let mut key_part_iter = key.iter();
        let mut key_part_idx = 0;

        while let Some(key_part) = key_part_iter.next() {
            key_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = unsafe { node.children.as_ref().unwrap_unchecked() };

                children
                    .iter()
                    .take_while(|child| unsafe {
                        child.key_part.as_ref().unwrap_unchecked().is_wildcard()
                    })
                    .for_each(|child| {
                        wildcards.push((key_part_idx, child));
                    });

                if let Ok(child_idx) = children.binary_search_by(|child| {
                    let child_key_part = unsafe { child.key_part.as_ref().unwrap_unchecked() };
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

    /// Find a value with matching wildcard part, and store captured matched wildcard parts in a map
    ///
    /// Return the smallest value that matches the given key
    pub fn find_and_capture<M: Captures<W, E>>(&self, key: &[E], captures: &mut M) -> Option<&V> {
        let mut node = &self.root;

        let mut wildcards = Vec::with_capacity(self.max_wildcard_depth);
        let mut captured = Vec::with_capacity(self.max_wildcard_depth);

        let mut key_part_iter = key.iter();
        let mut key_part_idx = 0;

        while let Some(key_part) = key_part_iter.next() {
            key_part_idx += 1;

            let mut try_backtrack = node.children.is_none();

            if !try_backtrack {
                let children = unsafe { node.children.as_ref().unwrap_unchecked() };

                children
                    .iter()
                    .take_while(|child| unsafe {
                        child.key_part.as_ref().unwrap_unchecked().is_wildcard()
                    })
                    .for_each(|child| {
                        wildcards.push((key_part_idx, child));
                    });

                if let Ok(child_idx) = children.binary_search_by(|child| {
                    let child_key_part = unsafe { child.key_part.as_ref().unwrap_unchecked() };
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
                    let discard_count = captured
                        .iter()
                        .rev()
                        .filter(|(captured_key_part_idx, _, _)| {
                            captured_key_part_idx >= &wildcard_key_part_idx
                        })
                        .count();

                    captured.truncate(captured.len() - discard_count);
                    captured.push((
                        wildcard_key_part_idx,
                        wildcard_node,
                        &key[wildcard_key_part_idx - 1],
                    ));

                    key_part_idx = wildcard_key_part_idx;
                    key_part_iter = key[wildcard_key_part_idx..].iter();
                    node = wildcard_node;
                } else {
                    return None;
                }
            }
        }

        for (_, node, matched_key_part) in captured.into_iter() {
            let wildcard_key_part = unsafe {
                node.key_part
                    .as_ref()
                    .unwrap_unchecked()
                    .as_ref()
                    .unwrap_wildcard()
            };
            captures.insert(wildcard_key_part.clone(), matched_key_part.clone());
        }

        node.value.as_ref()
    }

    /// Find a value without matching wildcard part
    pub fn find_exact(&self, key: &[E]) -> Option<&V> {
        let mut node = &self.root;

        for key_part in key {
            node.children.as_ref()?;

            let children = unsafe { node.children.as_ref().unwrap_unchecked() };

            if let Ok(child_idx) = children.binary_search_by(|child| {
                let child_key_part = unsafe { child.key_part.as_ref().unwrap_unchecked() };
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

impl<E: Debug, W: Debug, V: Debug> Debug for PrefixTreeMap<E, W, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "PrefixTreeMap {{ root: {:?} }}", self.root)
    }
}

impl<E: Debug, W: Debug, V: Debug> Debug for Node<E, W, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key_part.partial_cmp(&other.key_part)
    }
}

impl<E, W, V> Ord for Node<E, W, V>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.key_part.cmp(&other.key_part)
    }
}
