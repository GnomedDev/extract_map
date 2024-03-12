use std::iter::FusedIterator;

use crate::{value_wrapper::ValueWrapper, ExtractMap};

#[must_use = "Iterators do nothing if not consumed"]
pub struct Iter<'a, K, V>(pub(crate) std::collections::hash_set::Iter<'a, ValueWrapper<K, V>>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| &v.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K, V, S> IntoIterator for &'a ExtractMap<K, V, S> {
    type Item = &'a V;
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.inner.iter())
    }
}
