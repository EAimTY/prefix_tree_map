use crate::std_lib::BTreeMap;

#[cfg(feature = "std")]
use std::{collections::HashMap, hash::Hash};

/// The trait for customizing the capture map
pub trait Captures<W, E> {
    fn insert(&mut self, key: W, value: E);
}

impl<W, E> Captures<W, E> for BTreeMap<W, E>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }
}

#[cfg(feature = "std")]
impl<W, E> Captures<W, E> for HashMap<W, E>
where
    E: Clone + Ord,
    W: Clone + Hash + Ord,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }
}
