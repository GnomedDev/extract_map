use typesize::{if_typesize_details, TypeSize};

use crate::ExtractMap;

impl<K, V: TypeSize, S: TypeSize> TypeSize for ExtractMap<K, V, S> {
    fn extra_size(&self) -> usize {
        self.table.extra_size() + self.build_hasher.extra_size()
    }

    if_typesize_details! {
        fn get_collection_item_count(&self) -> Option<usize> {
            Some(self.len())
        }
    }
}
