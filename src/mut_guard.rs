use std::{
    hash::{BuildHasher, Hash},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use crate::{ExtractKey, ExtractMap};

pub struct MutGuard<'a, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    pub(crate) value: ManuallyDrop<V>,
    pub(crate) map: &'a mut ExtractMap<K, V, S>,
}

impl<K, V, S> Drop for MutGuard<'_, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    fn drop(&mut self) {
        // SAFETY: The ManuallyDrop is never used again as we are in Drop.
        let value = unsafe { ManuallyDrop::take(&mut self.value) };

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
        &self.value
    }
}

impl<K, V, S> DerefMut for MutGuard<'_, K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
