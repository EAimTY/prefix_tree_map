use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

/// The trait for customizing the capture map.
pub trait CaptureMap<W, E> {
    fn insert(&mut self, key: W, value: E);
    fn remove(&mut self, key: &W);
    fn clear(&mut self);
}

impl<W, E> CaptureMap<W, E> for BTreeMap<W, E>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }

    fn remove(&mut self, key: &W) {
        self.remove(key);
    }

    fn clear(&mut self) {
        self.clear();
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

    fn remove(&mut self, key: &W) {
        self.remove(key);
    }

    fn clear(&mut self) {
        self.clear();
    }
}
