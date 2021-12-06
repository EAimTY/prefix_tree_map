use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

/// The trait for customizing the capture map.
pub trait CaptureMap<W, E> {
    fn insert(&mut self, key: W, value: E);
}

impl<W, E> CaptureMap<W, E> for BTreeMap<W, E>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }
}

impl<W, E> CaptureMap<W, E> for HashMap<W, E>
where
    E: Clone + Ord,
    W: Clone + Hash + Ord,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }
}
