use crate::ExtractKey;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    id: u64,
    name: &'static str,
}

impl ExtractKey<u64> for User {
    fn extract_key(&self) -> &u64 {
        &self.id
    }
}
