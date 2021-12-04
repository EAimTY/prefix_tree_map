use std::{collections::{BTreeMap, HashMap}, hash::Hash};

pub trait CaptureMap<W, E> {
    fn insert(&mut self, key: W, value: E);
    fn remove(&mut self, key: &W);
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
}
