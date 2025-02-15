use std::{
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use crate::{ExtractKey, ExtractMap};

pub(crate) struct WithSizeHint<I> {
    inner: I,
    hint: Option<usize>,
}

impl<Item, I: Iterator<Item = Item>> Iterator for WithSizeHint<I> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.hint.unwrap_or_default(), self.hint)
    }
}

pub(crate) trait IteratorExt: Iterator + Sized {
    fn with_size_hint(self, hint: Option<usize>) -> WithSizeHint<Self>;
}

impl<Item, I: Iterator<Item = Item>> IteratorExt for I {
    fn with_size_hint(self, hint: Option<usize>) -> WithSizeHint<I> {
        WithSizeHint { inner: self, hint }
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
                let size_hint = map.size_hint();
                std::iter::from_fn(|| map.next_entry::<IgnoredAny, V>().transpose())
                    .map(|res| res.map(|(_, v)| v))
                    .with_size_hint(size_hint)
                    .collect()
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let size_hint = seq.size_hint();
                std::iter::from_fn(|| seq.next_element().transpose())
                    .with_size_hint(size_hint)
                    .collect()
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
