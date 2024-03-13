use extract_map::{ExtractKey, ExtractMap, LendingIterator};

struct User {
    id: u64,
    name: String,
}

impl ExtractKey<u64> for User {
    fn extract_key(&self) -> &u64 {
        &self.id
    }
}

#[test]
pub fn test() {
    let mut map = ExtractMap::<u64, User>::new();
    map.iter_mut().for_each(|_| {});
}
