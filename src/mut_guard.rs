use std::{
    hash::{BuildHasher, Hash},
    ops::{Deref, DerefMut},
};

use crate::{ExtractKey, ExtractMap};

pub struct MutGuard<'a, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    pub(crate) value: Option<V>,
    pub(crate) map: &'a mut ExtractMap<K, V, S>,
}

impl<K, V, S> Drop for MutGuard<'_, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    fn drop(&mut self) {
        let value = self.value.take().unwrap();
        self.map.insert(value);
    }
}

impl<K, V, S> Deref for MutGuard<'_, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<K, V, S> DerefMut for MutGuard<'_, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}
