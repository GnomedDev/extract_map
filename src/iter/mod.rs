#![allow(clippy::module_name_repetitions)]

use super::ExtractMap;
macro_rules! forward_iterator {
    (
        pub struct $ty_name:ident<$($lt:lifetime,)? V>($inner_ty:ty),
        $item:ty,
        |$var:ident: $map:ty| $inner:expr
    ) => {
        #[must_use = "Iterators do nothing if not consumed"]
        pub struct $ty_name<$($lt,)* V>($inner_ty);

        impl<$($lt,)* V: std::fmt::Debug> std::fmt::Debug for $ty_name<$($lt,)* V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl<$($lt,)* V> Iterator for $ty_name<$($lt,)* V> {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }
        }

        impl<$($lt,)* V> ExactSizeIterator for $ty_name<$($lt,)* V> {
            fn len(&self) -> usize {
                self.0.len()
            }
        }

        impl<$($lt,)* V> std::iter::FusedIterator for $ty_name<$($lt,)* V> {}

        impl<$($lt,)* K, V, S> IntoIterator for $map {
            type Item = $item;
            type IntoIter = $ty_name<$($lt,)* V>;

            fn into_iter(self) -> Self::IntoIter {
                $ty_name((|$var: $map|$inner)(self))
            }
        }
    };
}

forward_iterator!(
    pub struct IntoIter<V>(hashbrown::hash_table::IntoIter<V>),
    V,
    |map: ExtractMap<K, V, S>| map.table.into_iter()
);

forward_iterator!(
    pub struct Iter<'a, V>(hashbrown::hash_table::Iter<'a, V>),
    &'a V,
    |map: &'a ExtractMap<K, V, S>| map.table.iter()
);

forward_iterator!(
    pub struct IterMut<'a, V>(hashbrown::hash_table::IterMut<'a, V>),
    &'a mut V,
    |map: &'a mut ExtractMap<K, V, S>| map.table.iter_mut()
);
