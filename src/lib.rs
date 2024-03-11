#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use std::{
    fmt::Debug,
    hash::{BuildHasher, Hash, RandomState},
    marker::PhantomData,
};

use hashbrown::HashMap;

use value_wrapper::ValueWrapper;

pub mod iter;
mod value_wrapper;

pub trait ExtractKey<K> {
    fn extract_key(&self) -> &K;
}

/// A hash map for memory efficent storage of value types which contain their own keys.
///
/// This is achieved by the `V` type deriving the [`ExtractKey`] trait for their `K` type,
/// and is backed by a `hashbrown::HashMap<Wrap<K>, V, S>`, meaning this library is `#![forbid(unsafe_code)]`.
///
/// The default hashing algorithm is the same as the standard library's [`HashMap`], [`RandomState`],
/// although your own hasher can be provided via [`ExtractMap::with_hasher`] and it's similar methods.
#[cfg_attr(feature = "typesize", derive(typesize::TypeSize))]
pub struct ExtractMap<K, V, S = RandomState> {
    inner: HashMap<ValueWrapper<K, V>, (), S>,
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
            inner: HashMap::with_hasher(hasher),
        }
    }

    /// Creates a new [`ExtractMap`] with the provided hasher and preallocated capacity.
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
    /// let map = ExtractMap::<u64, User>::with_capacity_and_hasher(5, std::hash::RandomState::new());
    ///
    /// assert_eq!(map.len(), 0);
    /// assert!(map.capacity() >= 5);
    /// ```
    #[must_use]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            inner: HashMap::with_capacity_and_hasher(capacity, hasher),
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
    pub fn insert(&mut self, value: V) {
        self.inner.insert(ValueWrapper(value, PhantomData), ());
    }

    /// Retrieves a value from the [`ExtractMap`].
    ///
    /// This will retrieve the value based on the key extracted using [`ExtractKey`].
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.inner.hasher().hash_one(key);
        self.inner
            .raw_entry()
            .from_hash(hash, |v| v.0.extract_key() == key)
            .map(|(v, ())| &v.0)
    }
}

impl<K, V, S> ExtractMap<K, V, S> {
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

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

impl<K, V: Clone, S: Clone> Clone for ExtractMap<K, V, S> {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self { inner }
    }
}

impl<K, V, S> Debug for ExtractMap<K, V, S>
where
    K: Debug,
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
        use serde::de::{DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess};

        struct SeqMapAdapter<M>(M);

        impl<'de, M, E> SeqAccess<'de> for SeqMapAdapter<M>
        where
            E: Error,
            M: MapAccess<'de, Error = E>,
        {
            type Error = E;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: DeserializeSeed<'de>,
            {
                self.0
                    .next_entry_seed(PhantomData::<IgnoredAny>, seed)
                    .map(|o| o.map(|(_, v)| v))
            }
        }

        struct Visitor<K, V, S>(PhantomData<(K, V, S)>);

        impl<'de, K, V, S> serde::de::Visitor<'de> for Visitor<K, V, S>
        where
            K: Hash + Eq,
            V: ExtractKey<K> + serde::Deserialize<'de>,
            S: BuildHasher + Default,
        {
            type Value = ExtractMap<K, V, S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                self.visit_seq(SeqMapAdapter(map))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut map = ExtractMap::with_capacity_and_hasher(
                    seq.size_hint().unwrap_or_default(),
                    S::default(),
                );

                while let Some(elem) = seq.next_element()? {
                    map.insert(elem);
                }

                Ok(map)
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
