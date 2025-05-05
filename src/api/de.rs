use serde::de::{Expected, Unexpected};

pub fn deserialize_number_unconditionally<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl serde::de::Visitor<'_> for StringOrNumberVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or number")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.parse().unwrap())
        }
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

pub fn deserialize_map_with_empty_values_as_list_thats_actually_a_list_if_its_empty<'de, D>(
    deserializer: D,
) -> Result<Vec<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct MapOrListVisitor;

    impl<'de> serde::de::Visitor<'de> for MapOrListVisitor {
        type Value = Vec<u64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("map or list")
        }

        // Note: Always empty
        fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            Ok(vec![])
        }

        fn visit_map<A>(self, mut value: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut keys = vec![];
            while let Ok(Some((key, ()))) = value.next_entry() {
                keys.push(key);
            }
            Ok(keys)
        }
    }

    deserializer.deserialize_any(MapOrListVisitor)
}

pub fn strip_url_prefix<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StripUrlPrefixVisitor;

    impl serde::de::Visitor<'_> for StripUrlPrefixVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Valid URL")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let (prefix, path) = value.split_once(".net").unwrap_or(("", value));
            let prefix = match prefix {
                "https://i.pximg" => "imageproxy",
                "https://s.pximg" => "simg",
                "https://img-sketch.pixiv" => "spix",
                "https://img-sketch.pximg" => "spxi",
                prefix => {
                    return Err(E::invalid_value(
                        Unexpected::Str(prefix),
                        &ValidPixivImageUrl,
                    ))
                }
            };
            Ok(format!("/{prefix}{path}"))
        }
    }

    deserializer.deserialize_any(StripUrlPrefixVisitor)
}

struct ValidPixivImageUrl;

impl Expected for ValidPixivImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(
            "Valid Pixiv Image URL: i.pximg.net, img-sketch.pixiv.net, img-sketch.pximg.net",
        )
    }
}
