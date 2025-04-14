//! See [`ExtractMap`] for the main documentation.
//!
//! ## MSRV
//!
//! The Minimum Supported Rust Version for this crate is 1.70, and raising it is considered a breaking change.
#![warn(clippy::pedantic, rust_2018_idioms, missing_docs)]

use std::{
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher as _},
    marker::PhantomData,
    mem::{replace, ManuallyDrop},
};

use hashbrown::{hash_table::Entry, HashTable};
use mut_guard::MutGuard;

#[doc(hidden)]
pub mod iter;
mod mut_guard;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "typesize")]
mod typesize;

#[cfg(feature = "iter_mut")]
pub use gat_lending_iterator::LendingIterator;
#[cfg(feature = "serde")]
pub use serde::serialize_as_map;

fn hash_one<S: BuildHasher, H: Hash>(build_hasher: &S, val: H) -> u64 {
    let mut hasher = build_hasher.build_hasher();
    val.hash(&mut hasher);
    hasher.finish()
}

/// A trait for extracting the key for an [`ExtractMap`].
///
/// This is relied on for correctness in the same way as [`Hash`] and [`Eq`] are and
/// is purely designed for directly referencing a field with no interior mutability or
/// static return type.
pub trait ExtractKey<K: Hash + Eq> {
    /// Extracts the key that this value should be referred to with.
    fn extract_key(&self) -> &K;
}

/// A hash map for memory efficent storage of value types which contain their own keys.
///
/// This is backed by [`hashbrown::HashTable`], which is the backing storage for [`std`]'s [`HashSet`] and [`HashMap`].
///
/// The default hashing algorithm is the same as the standard library's hashing collections, [`RandomState`],
/// although your own hasher can be provided via [`ExtractMap::with_hasher`] and it's similar methods.
///
/// [`HashSet`]: std::collections::HashSet
/// [`HashMap`]: std::collections::HashMap
pub struct ExtractMap<K, V, S = RandomState> {
    // Any new fields added should be added to the `typesize` impl
    table: hashbrown::HashTable<V>,
    phantom: PhantomData<K>,
    build_hasher: S,
}

impl<K, V, S: Default> Default for ExtractMap<K, V, S> {
    fn default() -> Self {
        Self::with_hasher(S::default())
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
    pub fn with_hasher(hash_builder: S) -> Self {
        Self {
            table: HashTable::new(),
            phantom: PhantomData,
            build_hasher: hash_builder,
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
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self {
            table: HashTable::with_capacity(capacity),
            phantom: PhantomData,
            build_hasher: hash_builder,
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
        let key = value.extract_key();
        let entry = self.table.entry(
            hash_one(&self.build_hasher, key),
            |v| key == v.extract_key(),
            |v| hash_one(&self.build_hasher, v.extract_key()),
        );

        match entry {
            Entry::Occupied(entry) => Some(replace(entry.into_mut(), value)),
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
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
        let hash = hash_one(&self.build_hasher, key);
        let entry = self.table.find_entry(hash, |v| key == v.extract_key());

        match entry {
            Ok(entry) => Some(entry.remove().0),
            Err(_) => None,
        }
    }

    /// Checks if a value is in the [`ExtractMap`].
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Retrieves a value from the [`ExtractMap`].
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = hash_one(&self.build_hasher, key);
        self.table.find(hash, |v| key == v.extract_key())
    }

    /// Retrieves a mutable guard to a value in the [`ExtractMap`].
    ///
    /// This guard is required as the current implementation takes the value out
    /// of the map and reinserts on Drop to allow mutation of the key field.
    #[must_use]
    pub fn get_mut<'a>(&'a mut self, key: &K) -> Option<MutGuard<'a, K, V, S>> {
        let value = self.remove(key)?;
        Some(MutGuard {
            value: ManuallyDrop::new(value),
            map: self,
        })
    }
}

impl<K, V, S> ExtractMap<K, V, S> {
    /// Retrieves the number of remaining values that can be inserted before a reallocation.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.table.capacity()
    }

    /// Retrieves the number of values currently in the [`ExtractMap`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Retrieves if the [`ExtractMap`] contains no values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the total amount of memory allocated internally, in bytes.
    ///
    /// The returned number is informational only. It is intended to be
    /// primarily used for memory profiling.
    pub fn allocation_size(&self) -> usize {
        self.table.allocation_size()
    }

    /// Retrieves an iterator over the borrowed values.
    ///
    /// If you need an iterator over the keys and values, simply use [`ExtractKey`].
    ///
    /// Use [`IntoIterator::into_iter`] for an iterator over owned values.
    pub fn iter(&self) -> iter::Iter<'_, V> {
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
        Self {
            build_hasher: self.build_hasher.clone(),
            table: self.table.clone(),
            phantom: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.table.clone_from(&source.table);
        self.build_hasher.clone_from(&source.build_hasher);
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
        let iter = iter.into_iter();
        let mut this = Self::with_capacity_and_hasher(iter.size_hint().0, S::default());

        for value in iter {
            this.insert(value);
        }

        this
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
