use std::{borrow::Borrow, hash::Hash, marker::PhantomData};

use crate::ExtractKey;

pub(crate) struct ValueWrapper<K, V>(pub V, pub PhantomData<K>);

impl<K, V> Borrow<K> for ValueWrapper<K, V>
where
    V: ExtractKey<K>,
{
    fn borrow(&self) -> &K {
        self.0.extract_key()
    }
}

impl<K, V> Hash for ValueWrapper<K, V>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.extract_key().hash(state);
    }
}

impl<K, V> PartialEq for ValueWrapper<K, V>
where
    K: PartialEq,
    V: ExtractKey<K>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.extract_key() == other.0.extract_key()
    }
}

impl<K, V> Eq for ValueWrapper<K, V>
where
    K: Eq,
    V: ExtractKey<K>,
{
}

impl<K, V: Clone> Clone for ValueWrapper<K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}
