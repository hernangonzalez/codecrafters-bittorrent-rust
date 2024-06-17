use anyhow::{Context, Result};
use std::{collections::HashMap, fmt::Display, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum Ben {
    String(String),
    Number(i64),
    List(Vec<Ben>),
    Map(HashMap<String, Ben>),
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

impl FromStr for Ben {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let (r, b) = Self::decode(s)?;
        anyhow::ensure!(r.is_empty(), "Unexpected remains: {r}");
        Ok(b)
    }
}

impl Ben {
    fn decode_str(input: &str) -> Result<(&str, &str)> {
        let (count, input) = input.split_once(':').context("invalid string format")?;
        let count: usize = count.parse()?;
        anyhow::ensure!(count <= input.len(), "invalid ben string lenght: {count}");
        let (input, out) = input.split_at(count);
        Ok((out, input))
    }

    fn decode_string(input: &str) -> Result<(&str, Ben)> {
        Self::decode_str(input).map(|t| (t.0, Ben::String(t.1.to_owned())))
    }

    fn decode_number(input: &str) -> Result<(&str, Ben)> {
        let input = input.strip_prefix('i').context("invalid ben integer")?;
        let (input, out) = input.split_once('e').context("invalid ben integer")?;
        let ben = input.parse().map(Ben::Number)?;
        Ok((out, ben))
    }

    fn decode_list(input: &str) -> Result<(&str, Ben)> {
        let input = input.strip_prefix('l').context("invalid ben list start")?;
        let mut input = input;
        let mut v = vec![];
        while !input.starts_with('e') && !input.is_empty() {
            let ben;
            (input, ben) = Self::decode(input)?;
            v.push(ben);
        }
        let ben = Ben::List(v);
        anyhow::ensure!(!input.is_empty(), "Invalid end of list");
        let input = &input[1..];
        Ok((input, ben))
    }

    fn decode_dict(input: &str) -> Result<(&str, Ben)> {
        let mut input = input.strip_prefix('d').context("invalid ben dict start")?;
        let mut map = HashMap::<String, Ben>::new();
        while !input.starts_with('e') && !input.is_empty() {
            let key;
            let ben;
            (input, key) = Self::decode_str(input)?;
            (input, ben) = Self::decode(input)?;
            map.insert(key.to_owned(), ben);
        }
        anyhow::ensure!(!input.is_empty(), "Invalid end of dict");
        let input = &input[1..];
        let ben = Self::Map(map);
        Ok((input, ben))
    }

    fn decode(input: &str) -> Result<(&str, Ben)> {
        match input.chars().next().context("Input is empty")? {
            c if c.is_ascii_digit() => Self::decode_string(input),
            'i' => Self::decode_number(input),
            'l' => Self::decode_list(input),
            'd' => Self::decode_dict(input),
            _ => anyhow::bail!("Unknown encoding: {input}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string() {
        let b: Ben = "9:hernan.rs".parse().unwrap();
        assert_eq!(b, "hernan.rs");
        assert_eq!(b.to_string(), r#""hernan.rs""#);
    }

    #[test]
    fn test_integer() {
        let b: Ben = "i42e".parse().unwrap();
        assert_eq!(b, 42);

        let b: Ben = "i-42e".parse().unwrap();
        assert_eq!(b, -42);
    }

    #[test]
    fn test_list() {
        let b: Ben = "l5:helloi52ee".parse().unwrap();
        assert_eq!(b.to_string(), r#"["hello",52]"#);

        let b: Ben = "lli4eei5ee".parse().unwrap();
        assert_eq!(b.to_string(), "[[4],5]");
    }

    #[test]
    fn test_dict() {
        let b: Ben = "d3:foo3:bar5:helloi52ee".parse().unwrap();
        assert_eq!(b.to_string(), r#"{"foo":"bar","hello":52}"#);
    }
}
