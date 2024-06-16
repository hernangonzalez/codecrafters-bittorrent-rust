use anyhow::{Context, Result};
use std::{fmt::Display, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub enum Ben {
    String(String),
    Number(i64),
    List(Vec<Ben>),
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
        match value {
            Ben::String(s) => serde_json::Value::String(s.clone()),
            Ben::Number(i) => serde_json::Value::from(*i),
            Ben::List(v) => {
                serde_json::Value::Array(v.iter().map(serde_json::Value::from).collect())
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
    fn decode(input: &str) -> Result<(&str, Ben)> {
        match input.chars().next() {
            Some(c) if c.is_ascii_digit() => {
                let (count, input) = input.split_once(':').context("invalid string format")?;
                let count: usize = count.parse()?;
                anyhow::ensure!(count <= input.len(), "invalid ben string lenght: {count}");
                let (input, out) = input.split_at(count);
                let ben = Ben::String(input.to_owned());
                Ok((out, ben))
            }
            Some('i') => {
                let input = input.strip_prefix('i').context("invalid ben integer")?;
                let (input, out) = input.split_once('e').context("invalid ben integer")?;
                let ben = input.parse().map(Ben::Number)?;
                Ok((out, ben))
            }
            Some('l') => {
                let input = input
                    .strip_prefix('l')
                    .context("invalid ben list start: {input}")?;
                let mut input = input;
                let mut ben;
                let mut v = vec![];
                while !input.starts_with('e') {
                    (input, ben) = Self::decode(input)?;
                    v.push(ben);
                }
                let ben = Ben::List(v);
                let input = &input[1..];
                Ok((input, ben))
            }
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
        assert_eq!(b.to_string(), "\"hernan.rs\"");
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
        assert_eq!(b.to_string(), "[\"hello\",52]");

        let b: Ben = "lli4eei5ee".parse().unwrap();
        assert_eq!(b.to_string(), "[[4],5]");
    }
}
