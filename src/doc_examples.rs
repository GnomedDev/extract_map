use crate::ExtractKey;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: &'static str,
}

impl ExtractKey<u64> for User {
    fn extract_key(&self) -> &u64 {
        &self.id
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for User {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        struct Visitor;

        impl<'a> serde::de::Visitor<'a> for Visitor {
            type Value = User;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a User struct")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'a>,
            {
                let mut id = None::<u64>;
                let mut name = None::<Box<str>>;

                let mut run_deser_pass = || {
                    match map.next_key()? {
                        Some("id") => id = Some(map.next_value()?),
                        Some("name") => name = Some(map.next_value()?),
                        None => return Err(Error::missing_field("id")),
                        Some(_) => {}
                    }

                    Ok(())
                };

                run_deser_pass()?;
                run_deser_pass()?;

                Ok(User {
                    id: id.ok_or_else(|| Error::missing_field("id"))?,
                    name: Box::leak(name.ok_or_else(|| Error::missing_field("name"))?),
                })
            }
        }

        deserializer.deserialize_struct("User", &["id", "name"], Visitor)
    }
}
