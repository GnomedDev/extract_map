use std::iter::FusedIterator;

use crate::ExtractMap;

#[must_use = "Iterators do nothing if not consumed"]
pub struct Iter<'a, V>(pub(crate) hashbrown::hash_table::Iter<'a, V>);

// impl<V> Clone for Iter<'_, V> {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl<V: std::fmt::Debug> std::fmt::Debug for Iter<'_, V> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_list().entries(self.clone()).finish()
//     }
// }

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<V> ExactSizeIterator for Iter<'_, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<V> FusedIterator for Iter<'_, V> {}

impl<'a, K, V, S> IntoIterator for &'a ExtractMap<K, V, S> {
    type Item = &'a V;
    type IntoIter = Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.table.iter())
    }
}
