use std::iter::FusedIterator;

use crate::{value_wrapper::ValueWrapper, ExtractMap};

#[must_use = "Iterators do nothing if not consumed"]
pub struct IntoIter<K, V>(hashbrown::hash_map::IntoKeys<ValueWrapper<K, V>, ()>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<K, V, S> IntoIterator for ExtractMap<K, V, S> {
    type Item = V;
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.inner.into_keys())
    }
}
