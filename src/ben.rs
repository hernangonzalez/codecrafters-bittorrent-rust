use std::{collections::HashMap, fmt::Display};

pub type List = Vec<Ben>;
pub type Map = HashMap<String, Ben>;

#[derive(Clone, Debug, PartialEq)]
pub enum Ben {
    String(String),
    Number(i64),
    List(List),
    Map(Map),
}

impl PartialEq<i64> for Ben {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Ben::Number(i) => i == other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Ben {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Ben::String(s) => s == other,
            _ => false,
        }
    }
}

impl From<&Ben> for serde_json::Value {
    fn from(value: &Ben) -> Self {
        use serde_json::Value;
        match value {
            Ben::String(s) => Value::String(s.clone()),
            Ben::Number(i) => Value::from(*i),
            Ben::List(v) => Value::Array(v.iter().map(Value::from).collect()),
            Ben::Map(m) => {
                let map = m.iter().map(|x| (x.0.to_owned(), Self::from(x.1))).fold(
                    serde_json::Map::new(),
                    |mut map, x| {
                        map.insert(x.0, x.1);
                        map
                    },
                );
                Value::Object(map)
            }
        }
    }
}

impl Display for Ben {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = serde_json::Value::from(self);
        write!(f, "{value}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display_string() {
        let b = Ben::String("hernan.rs".into());
        assert_eq!(b.to_string(), r#""hernan.rs""#);
    }

    #[test]
    fn test_display_integer() {
        let b = Ben::Number(42);
        assert_eq!(b.to_string(), "42");

        let b = Ben::Number(-42);
        assert_eq!(b.to_string(), "-42");
    }

    #[test]
    fn test_display_list() {
        let b = Ben::List(vec![Ben::String("hello".into()), Ben::Number(82)]);
        assert_eq!(b.to_string(), r#"["hello",82]"#);
    }

    #[test]
    fn test_display_dict() {
        let mut m: Map = Map::new();
        m.insert("foo".into(), Ben::String("bar".into()));
        m.insert("hello".into(), Ben::Number(82));
        let b: Ben = Ben::Map(m);
        assert_eq!(b.to_string(), r#"{"foo":"bar","hello":82}"#);
    }
}
