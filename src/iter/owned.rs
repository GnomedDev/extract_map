use std::iter::FusedIterator;

use crate::ExtractMap;

#[must_use = "Iterators do nothing if not consumed"]
pub struct IntoIter<V>(hashbrown::hash_table::IntoIter<V>);

impl<V: std::fmt::Debug> std::fmt::Debug for IntoIter<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl<V> Iterator for IntoIter<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<V> ExactSizeIterator for IntoIter<V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<V> FusedIterator for IntoIter<V> {}

impl<K, V, S> IntoIterator for ExtractMap<K, V, S> {
    type Item = V;
    type IntoIter = IntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.table.into_iter())
    }
}
