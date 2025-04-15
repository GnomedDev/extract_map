#[cfg(doc)]
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
    /// map.insert(User { id; 1, name: "Cat" });
    ///
    /// let entry = map.entry(1).insert(User { id: 1, name: "Fox" });
    /// assert_eq!(entry.get(), User { id: 1, name: "Fox" });
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
    /// let entry = map.entry(1).or_insert(User { id: 1, name: "Fox" });
    /// assert_eq!(entry.get(), User { id: 1, name: "Fox" });
    ///
    /// // Does not insert new entry, as there is already a user with ID 1.
    /// let entry = map.entry(1).or_insert(User { id: 1, name: "Cat" });
    /// assert_eq!(entry.get(), User { id: 1, name: "Fox" });
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
    /// let entry = map.entry(1).or_insert_with(|| User { id: 1, name: "Fox" });
    /// assert_eq!(entry.get(), User { id: 1, name: "Fox" });
    ///
    /// // Does not insert new entry, as there is already a user with ID 1.
    /// let entry = map.entry(1).or_insert_with(|| User { id: 1, name: "Cat" });
    /// assert_eq!(entry.get(), User { id: 1, name: "Fox" });
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
    /// map.entry(1).and_modify(|user| user.name = "Fox");
    ///
    /// assert_eq!(map.get(1), User { id: 1, name: "Fox"});
    /// ```
    #[expect(clippy::return_self_not_must_use)]
    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        Self::from_raw(self.into_raw().and_modify(f))
    }
}

/// A view into an occupied entry in an [`ExtractMap`]. It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, V>(RawOccupiedEntry<'a, V>);

forward_debug!(OccupiedEntry);

impl<'a, V> OccupiedEntry<'a, V> {
    pub fn remove(self) -> (V, VacantEntry<'a, V>) {
        let (value, raw_vacant) = self.0.remove();
        (value, VacantEntry(raw_vacant))
    }

    pub fn get(&self) -> &V {
        self.0.get()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.0.get_mut()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.0.into_mut()
    }
}

pub struct VacantEntry<'a, V>(RawVacantEntry<'a, V>);

forward_debug!(VacantEntry);

impl<'a, V> VacantEntry<'a, V> {
    pub fn insert(self, value: V) -> OccupiedEntry<'a, V> {
        OccupiedEntry(self.0.insert(value))
    }
}
