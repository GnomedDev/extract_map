//! See [`ExtractMap`] for the main documentation.
//!
//! ## MSRV
//!
//! The Minimum Supported Rust Version for this crate is 1.70, and raising it is considered a breaking change.
#![warn(clippy::pedantic, rust_2018_idioms, missing_docs)]

use std::{
    collections::{hash_map::RandomState, HashSet},
    fmt::Debug,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    mem::ManuallyDrop,
};

use mut_guard::MutGuard;
use value_wrapper::ValueWrapper;

#[doc(hidden)]
pub mod iter;
mod mut_guard;
mod value_wrapper;

#[cfg(feature = "iter_mut")]
pub use gat_lending_iterator::LendingIterator;

/// A trait for extracting the key for an [`ExtractMap`].
///
/// This is relied on for correctness in the same way as [`Hash`] and [`Eq`] are and
/// is purely designed for directly referencing a field with no interior mutability or
/// static return type, the documentation on [`HashSet`] should be followed for this key type.
pub trait ExtractKey<K: Hash + Eq> {
    /// Extracts the key that this value should be referred to with.
    fn extract_key(&self) -> &K;
}

/// A hash map for memory efficent storage of value types which contain their own keys.
///
/// This is achieved by the `V` type deriving the [`ExtractKey`] trait for their `K` type,
/// and is backed by a `HashSet<Wrap<K>, V, S>`, meaning this library only uses unsafe code
/// for performance reasons.
///
/// The default hashing algorithm is the same as the standard library's [`HashSet`], [`RandomState`],
/// although your own hasher can be provided via [`ExtractMap::with_hasher`] and it's similar methods.
#[cfg_attr(feature = "typesize", derive(typesize::TypeSize))]
pub struct ExtractMap<K, V, S = RandomState> {
    inner: HashSet<ValueWrapper<K, V>, S>,
}

impl<K, V> Default for ExtractMap<K, V, RandomState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> ExtractMap<K, V, RandomState> {
    /// Creates a new, empty [`ExtractMap`] with the [`RandomState`] hasher.
    #[must_use]
    pub fn new() -> Self {
        Self::with_hasher(RandomState::new())
    }

    /// Creates a new [`ExtractMap`] with the [`RandomState`] hasher and preallocated capacity.
    ///
    /// # Examples
    /// ```
    /// use extract_map::{ExtractMap, ExtractKey};
    ///
    /// struct User {
    ///     id: u64,
    ///     name: &'static str,
    /// }
    ///
    /// impl ExtractKey<u64> for User {
    ///     fn extract_key(&self) -> &u64 {
    ///         &self.id
    ///     }
    /// }
    ///
    /// let map = ExtractMap::<u64, User>::with_capacity(5);
    ///
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 5);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, RandomState::new())
    }
}

impl<K, V, S> ExtractMap<K, V, S> {
    /// Creates a new, empty [`ExtractMap`] with the provided hasher.
    #[must_use]
    pub fn with_hasher(hasher: S) -> Self {
        Self {
            inner: HashSet::with_hasher(hasher),
        }
    }

    /// Creates a new [`ExtractMap`] with the provided hasher and preallocated capacity.
    ///
    /// # Examples
    /// ```
    /// use std::collections::hash_map::RandomState;
    ///
    /// use extract_map::{ExtractMap, ExtractKey};
    ///
    /// struct User {
    ///     id: u64,
    ///     name: &'static str,
    /// }
    ///
    /// impl ExtractKey<u64> for User {
    ///     fn extract_key(&self) -> &u64 {
    ///         &self.id
    ///     }
    /// }
    ///
    /// let map = ExtractMap::<u64, User>::with_capacity_and_hasher(5, RandomState::new());
    ///
    /// assert!(map.is_empty());
    /// assert!(map.capacity() >= 5);
    /// ```
    #[must_use]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            inner: HashSet::with_capacity_and_hasher(capacity, hasher),
        }
    }
}

impl<K, V, S> ExtractMap<K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    /// Inserts a value into the [`ExtractMap`].
    ///
    /// This extracts the key from the value using the [`ExtractKey`] trait, and therefore does not need a key to be provided.
    ///
    /// # Examples
    /// ```
    /// use extract_map::{ExtractMap, ExtractKey};
    ///
    /// struct User {
    ///     id: u64,
    ///     name: &'static str,
    /// }
    ///
    /// impl ExtractKey<u64> for User {
    ///     fn extract_key(&self) -> &u64 {
    ///         &self.id
    ///     }
    /// }
    ///
    /// let mut map = ExtractMap::new();
    /// map.insert(User { id: 1, name: "Daisy" });
    /// map.insert(User { id: 2, name: "Elliott" });
    ///
    /// assert_eq!(map.len(), 2);
    /// ```
    pub fn insert(&mut self, value: V) -> Option<V> {
        self.inner
            .replace(ValueWrapper(value, PhantomData))
            .map(|v| v.0)
    }

    /// Removes a value from the [`ExtractMap`].
    ///
    /// # Examples
    /// ```
    /// use extract_map::{ExtractMap, ExtractKey};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct User {
    ///     id: u64,
    ///     name: &'static str,
    /// }
    ///
    /// impl ExtractKey<u64> for User {
    ///     fn extract_key(&self) -> &u64 {
    ///         &self.id
    ///     }
    /// }
    ///
    /// let user = User { id: 1, name: "Daisy" };
    /// let mut map = ExtractMap::new();
    /// map.insert(user.clone());
    ///
    /// assert_eq!(map.remove(&1), Some(user));
    /// assert!(map.is_empty())
    /// ```
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.take(key).map(|v| v.0)
    }

    /// Checks if a value is in the [`ExtractMap`].
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.contains(key)
    }

    /// Retrieves a value from the [`ExtractMap`].
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key).map(|v| &v.0)
    }

    /// Retrieves a mutable guard to a value in the [`ExtractMap`].
    ///
    /// This guard is required as the current implementation takes the value out
    /// of the map and reinserts on Drop to allow mutation of the key field.
    #[must_use]
    pub fn get_mut<'a>(&'a mut self, key: &K) -> Option<MutGuard<'a, K, V, S>> {
        let value = self.inner.take(key)?;
        Some(MutGuard {
            value: ManuallyDrop::new(value.0),
            map: self,
        })
    }
}

impl<K, V, S> ExtractMap<K, V, S> {
    /// Retrieves the number of remaining values that can be inserted before a reallocation.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Retrieves the number of values currently in the [`ExtractMap`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Retrieves if the [`ExtractMap`] contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Retrieves an iterator over the borrowed values.
    ///
    /// If you need an iterator over the keys and values, simply use [`ExtractKey`].
    ///
    /// Use [`IntoIterator::into_iter`] for an iterator over owned values.
    pub fn iter(&self) -> iter::Iter<'_, K, V> {
        self.into_iter()
    }
}

#[cfg(feature = "iter_mut")]
impl<K, V, S> ExtractMap<K, V, S>
where
    K: Hash + Eq + Clone,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    /// Retrieves a [`LendingIterator`] over mutable borrowed values.
    ///
    /// This cannot implement [`Iterator`], so uses the `gat_lending_iterator` crate and has the
    /// performance cost of allocating a [`Vec`] of the keys cloned, so if possible should be avoided.
    ///
    /// To use, [`LendingIterator`] must be in scope, therefore this crate re-exports it.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter_mut(&mut self) -> iter::IterMut<'_, K, V, S> {
        iter::IterMut::new(self)
    }
}

impl<K, V: Clone, S: Clone> Clone for ExtractMap<K, V, S> {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<K, V, S> Debug for ExtractMap<K, V, S>
where
    K: Debug + Hash + Eq,
    V: Debug + ExtractKey<K>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|v| (v.extract_key(), v)))
            .finish()
    }
}

impl<K, V, S> PartialEq for ExtractMap<K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K> + PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|v| {
            let k = v.extract_key();

            other.get(k).is_some_and(|other_v| {
                let other_k = other_v.extract_key();
                k == other_k && v == other_v
            })
        })
    }
}

impl<K, V, S> FromIterator<V> for ExtractMap<K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let inner = iter
            .into_iter()
            .map(|item| ValueWrapper(item, PhantomData))
            .collect();

        Self { inner }
    }
}

impl<K, V, S> Extend<V> for ExtractMap<K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = V>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item);
        }
    }
}

/// Deserializes an [`ExtractMap`] from either a sequence or a map.
///
/// This uses [`serde::Deserializer::deserialize_any`], so may fail for formats which are not self-describing.
///
/// # Example
/// ```
/// use extract_map::{ExtractMap, ExtractKey};
///
/// #[derive(Debug, PartialEq, serde::Deserialize)]
/// struct User {
///     id: u64,
///     name: &'static str,
/// }
///
/// impl ExtractKey<u64> for User {
///     fn extract_key(&self) -> &u64 {
///         &self.id
///     }
/// }
///
/// let map_json = r#"{"0": {"id": 0, "name": "Elliott"}, "1": {"id": 1, "name": "Daisy"}}"#;
/// let seq_json = r#"[{"id": 0, "name": "Elliott"}, {"id": 1, "name": "Daisy"}]"#;
///
/// let map: ExtractMap<u64, User> = serde_json::from_str(map_json).unwrap();
/// let seq: ExtractMap<u64, User> = serde_json::from_str(seq_json).unwrap();
///
/// assert_eq!(map, seq);
/// ```
#[cfg(feature = "serde")]
impl<'de, K, V, S> serde::Deserialize<'de> for ExtractMap<K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K> + serde::Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{IgnoredAny, MapAccess, SeqAccess};

        struct Visitor<K, V, S>(PhantomData<(K, V, S)>);

        impl<'de, K, V, S> serde::de::Visitor<'de> for Visitor<K, V, S>
        where
            K: Hash + Eq,
            V: ExtractKey<K> + serde::Deserialize<'de>,
            S: BuildHasher + Default,
        {
            type Value = ExtractMap<K, V, S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                std::iter::from_fn(|| map.next_entry::<IgnoredAny, V>().transpose())
                    .map(|res| res.map(|(_, v)| v))
                    .collect()
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                std::iter::from_fn(|| seq.next_element().transpose()).collect()
            }
        }

        deserializer.deserialize_any(Visitor(PhantomData))
    }
}

/// Serializes an [`ExtractMap`] into a sequence of the values.
#[cfg(feature = "serde")]
impl<K, V: serde::Serialize, H> serde::Serialize for ExtractMap<K, V, H> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self)
    }
}

/// A serialize method to serialize a [`ExtractMap`] to a map instead of a sequence.
///
/// This should be used via serde's `serialize_with` field attribute.
///
/// # Errors
/// Errors if the underlying key or value serialisation fails.
#[cfg(feature = "serde")]
pub fn serialize_as_map<K, V, H, S>(map: &ExtractMap<K, V, H>, ser: S) -> Result<S::Ok, S::Error>
where
    K: serde::Serialize + Hash + Eq,
    V: serde::Serialize + ExtractKey<K>,
    S: serde::Serializer,
{
    ser.collect_map(map.iter().map(|v| (v.extract_key(), v)))
}
