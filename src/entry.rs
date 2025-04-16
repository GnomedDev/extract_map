//! An implementation of the Entry API for [`ExtractMap`].

use std::hash::{BuildHasher, Hash};

use crate::ExtractKey;

use super::ExtractMap;
use hashbrown::hash_table::{
    Entry as RawEntry, OccupiedEntry as RawOccupiedEntry, VacantEntry as RawVacantEntry,
};

macro_rules! forward_debug {
    ($type_name:ident) => {
        impl<'a, V: std::fmt::Debug> std::fmt::Debug for $type_name<'a, V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

impl<K, V, S> ExtractMap<K, V, S>
where
    K: Hash + Eq,
    V: ExtractKey<K>,
    S: BuildHasher,
{
    /// Gets the given key’s corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: &K) -> Entry<'_, V> {
        Entry::from_raw(self.raw_entry(key))
    }
}

/// A view into a single entry in a table, which may either be vacant or occupied.
///
/// This enum is constructed from [`ExtractMap::entry`].
#[derive(Debug)]
pub enum Entry<'a, V> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, V>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, V>),
}

impl<'a, V> Entry<'a, V> {
    fn from_raw(raw: RawEntry<'a, V>) -> Self {
        match raw {
            RawEntry::Occupied(raw_entry) => Entry::Occupied(OccupiedEntry(raw_entry)),
            RawEntry::Vacant(raw_entry) => Entry::Vacant(VacantEntry(raw_entry)),
        }
    }

    fn into_raw(self) -> RawEntry<'a, V> {
        match self {
            Entry::Occupied(entry) => RawEntry::Occupied(entry.0),
            Entry::Vacant(entry) => RawEntry::Vacant(entry.0),
        }
    }

    /// Sets the value of the entry, replacing any existing value if there is one, and returns an [`OccupiedEntry`].
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::ExtractMap;
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    /// map.insert(User { id: 1, name: "Cat" });
    ///
    /// let entry = map.entry(&1).insert(User { id: 1, name: "Fox" });
    /// assert_eq!(entry.get(), &User { id: 1, name: "Fox" });
    /// ```
    pub fn insert(self, value: V) -> OccupiedEntry<'a, V> {
        OccupiedEntry(self.into_raw().insert(value))
    }

    /// Ensures a value is in the entry by inserting if it was vacant.
    ///
    /// Returns an [`OccupiedEntry`] pointing to the now-occupied entry.
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::ExtractMap;
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    ///
    /// // Inserts new entry, as the map is empty.
    /// let entry = map.entry(&1).or_insert(User { id: 1, name: "Fox" });
    /// assert_eq!(entry.get(), &User { id: 1, name: "Fox" });
    ///
    /// // Does not insert new entry, as there is already a user with ID 1.
    /// let entry = map.entry(&1).or_insert(User { id: 1, name: "Cat" });
    /// assert_eq!(entry.get(), &User { id: 1, name: "Fox" });
    /// ```
    pub fn or_insert(self, default: V) -> OccupiedEntry<'a, V> {
        OccupiedEntry(self.into_raw().or_insert(default))
    }

    /// Ensures a value is in the entry by inserting the result of the function if it was vacant.
    ///
    /// Returns an [`OccupiedEntry`] pointing to the now-occupied entry.
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::ExtractMap;
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    ///
    /// // Inserts new entry, as the map is empty.
    /// let entry = map.entry(&1).or_insert_with(|| User { id: 1, name: "Fox" });
    /// assert_eq!(entry.get(), &User { id: 1, name: "Fox" });
    ///
    /// // Does not insert new entry, as there is already a user with ID 1.
    /// let entry = map.entry(&1).or_insert_with(|| User { id: 1, name: "Cat" });
    /// assert_eq!(entry.get(), &User { id: 1, name: "Fox" });
    /// ```
    pub fn or_insert_with(self, default: impl FnOnce() -> V) -> OccupiedEntry<'a, V> {
        OccupiedEntry(self.into_raw().or_insert_with(default))
    }

    /// Provides in-place mutable access to an occupied entry, does nothing for a vacant entry.
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::ExtractMap;
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    ///
    /// map.insert(User { id: 1, name: "Cat"});
    /// map.entry(&1).and_modify(|user| user.name = "Fox");
    ///
    /// assert_eq!(map.get(&1), Some(&User { id: 1, name: "Fox"}));
    /// ```
    #[allow(clippy::return_self_not_must_use)]
    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        Self::from_raw(self.into_raw().and_modify(f))
    }
}

/// A view into an occupied entry in an [`ExtractMap`]. It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, V>(RawOccupiedEntry<'a, V>);

forward_debug!(OccupiedEntry);

impl<'a, V> OccupiedEntry<'a, V> {
    /// Removes the value from the map.
    ///
    /// # Example
    /// ```
    /// use extract_map::{ExtractMap, entry::Entry};
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    /// map.insert(User { id: 1, name: "Fox" });
    ///
    /// if let Entry::Occupied(entry) = map.entry(&1) {
    ///     entry.remove();
    /// }
    ///
    /// assert!(map.is_empty());
    /// ```
    #[allow(clippy::must_use_candidate)]
    pub fn remove(self) -> V {
        self.0.remove().0
    }

    /// Gets a reference to the value from the map.
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::{ExtractMap, entry::Entry};
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    /// map.insert(User { id: 1, name: "Cat" });
    ///
    /// if let Entry::Occupied(entry) = map.entry(&1) {
    ///     assert_eq!(entry.get(), &User { id: 1, name: "Cat" });
    /// }
    /// ```
    #[must_use]
    pub fn get(&self) -> &V {
        self.0.get()
    }

    /// Gets a mutable reference to the value from the map.
    ///
    /// If you need a mutable reference borrowing from the map, instead of the entry, use [`Self::into_mut`].
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::{ExtractMap, entry::Entry};
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    /// map.insert(User { id: 1, name: "Cat" });
    ///
    /// if let Entry::Occupied(mut entry) = map.entry(&1) {
    ///     entry.get_mut().name = "Fox";
    /// }
    ///
    /// assert_eq!(map.get(&1), Some(&User { id: 1, name: "Fox" }));
    /// ```
    pub fn get_mut(&mut self) -> &mut V {
        self.0.get_mut()
    }

    /// Converts the [`OccupiedEntry`] into a mutable reference to the value from the map.
    ///
    /// If you need multiple mutable references to the entry, use [`Self::get_mut`].
    ///
    /// # Example
    ///
    /// ```
    /// use extract_map::{ExtractMap, entry::Entry};
    /// # use extract_map::doc_examples::User;
    ///
    /// let mut map: ExtractMap<u64, User> = ExtractMap::new();
    /// map.insert(User { id: 1, name: "Cat" });
    ///
    /// let user_ref = if let Entry::Occupied(entry) = map.entry(&1) {
    ///     entry.into_mut()
    /// } else {
    ///     unreachable!()
    /// };
    ///
    /// user_ref.name = "Fox";
    /// assert_eq!(map.get(&1), Some(&User { id: 1, name: "Fox" }));
    /// ```
    #[must_use]
    pub fn into_mut(self) -> &'a mut V {
        self.0.into_mut()
    }

    /// Sets the value of the entry, and returns the entry’s old value.
    ///
    /// This is equivalent to [`std::mem::replace`] with [`Self::get_mut`].
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.0.get_mut(), value)
    }
}

/// A view into a vacant entry in an [`ExtractMap`]. It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, V>(RawVacantEntry<'a, V>);

forward_debug!(VacantEntry);

impl<'a, V> VacantEntry<'a, V> {
    /// Sets the value of the entry with the [`VacantEntry`]’s key, and returns an [`OccupiedEntry`].
    pub fn insert(self, value: V) -> OccupiedEntry<'a, V> {
        OccupiedEntry(self.0.insert(value))
    }
}
