use std::{
    collections::VecDeque,
    hash::{BuildHasher, Hash},
};

use gat_lending_iterator::LendingIterator;

use crate::{mut_guard::MutGuard, ExtractKey, ExtractMap};

#[must_use = "Iterators do nothing if not consumed"]
pub struct IterMut<'a, K, V, S> {
    map: &'a mut ExtractMap<K, V, S>,
    keys: VecDeque<K>,
}

impl<K, V: std::fmt::Debug, S> std::fmt::Debug for IterMut<'_, K, V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.map.iter()).finish()
    }
}

impl<'a, K, V, S> IterMut<'a, K, V, S>
where
    K: Hash + Eq + Clone,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    pub fn new(map: &'a mut ExtractMap<K, V, S>) -> Self {
        let keys = map.iter().map(ExtractKey::extract_key).cloned().collect();
        Self { map, keys }
    }
}

impl<'map, K, V, S> LendingIterator for IterMut<'map, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    type Item<'item> = MutGuard<'item, K, V, S> where Self: 'item;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let key = self.keys.pop_front()?;
        self.map.get_mut(&key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.keys.len(), Some(self.keys.len()))
    }
}
