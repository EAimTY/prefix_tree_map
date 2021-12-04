use std::collections::BTreeMap;

pub trait CaptureMap<W, E> {
    fn insert(&mut self, key: W, value: E);
    fn remove(&mut self, key: &W);
}

impl<W, E> CaptureMap<W, E> for BTreeMap<W, E>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn insert(&mut self, key: W, value: E) {
        self.insert(key, value);
    }

    fn remove(&mut self, key: &W) {
        self.remove(key);
    }
}
