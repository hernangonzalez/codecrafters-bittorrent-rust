use crate::ben::{Ben, List, Map};
use anyhow::{Context, Result};
use std::{collections::HashMap, str::FromStr};

impl FromStr for Ben {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let (r, b) = Self::decode(s)?;
        anyhow::ensure!(r.is_empty(), "Unexpected remains: {r}");
        Ok(b)
    }
}

trait Decode
where
    Self: Sized,
{
    fn decode(input: &'_ str) -> Result<(&'_ str, Self)>;
}

impl Decode for String {
    fn decode(input: &str) -> Result<(&str, String)> {
        let (count, input) = input.split_once(':').context("invalid string format")?;
        let count: usize = count.parse()?;
        anyhow::ensure!(count <= input.len(), "invalid ben string lenght: {count}");
        let (input, out) = input.split_at(count);
        Ok((out, input.to_owned()))
    }
}

impl Decode for i64 {
    fn decode(input: &str) -> Result<(&str, i64)> {
        let input = input.strip_prefix('i').context("invalid ben integer")?;
        let (input, out) = input.split_once('e').context("invalid ben integer")?;
        let num = input.parse()?;
        Ok((out, num))
    }
}

impl Decode for Vec<Ben> {
    fn decode(input: &str) -> Result<(&str, Vec<Ben>)> {
        let input = input.strip_prefix('l').context("invalid ben list start")?;
        let mut input = input;
        let mut v = vec![];
        while !input.starts_with('e') && !input.is_empty() {
            let ben;
            (input, ben) = Ben::decode(input)?;
            v.push(ben);
        }
        anyhow::ensure!(!input.is_empty(), "Invalid end of list");
        let input = &input[1..];
        Ok((input, v))
    }
}

impl Decode for Map {
    fn decode(input: &str) -> Result<(&str, HashMap<String, Ben>)> {
        let mut input = input.strip_prefix('d').context("invalid ben dict start")?;
        let mut map = HashMap::<String, Ben>::new();
        while !input.starts_with('e') && !input.is_empty() {
            let key;
            let ben;
            (input, key) = String::decode(input)?;
            (input, ben) = Ben::decode(input)?;
            map.insert(key, ben);
        }
        anyhow::ensure!(!input.is_empty(), "Invalid end of dict");
        let input = &input[1..];
        Ok((input, map))
    }
}

impl Decode for Ben {
    fn decode(input: &str) -> Result<(&str, Ben)> {
        match input.chars().next().context("Input is empty")? {
            c if c.is_ascii_digit() => String::decode(input).map(|t| (t.0, Ben::String(t.1))),
            'i' => i64::decode(input).map(|t| (t.0, Ben::Number(t.1))),
            'l' => List::decode(input).map(|t| (t.0, Ben::List(t.1))),
            'd' => Map::decode(input).map(|t| (t.0, Ben::Map(t.1))),
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
        let b: Ben = "lli4eei5ee".parse().unwrap();
        let l = Ben::List(vec![Ben::List(vec![Ben::Number(4)]), Ben::Number(5)]);
        assert_eq!(b, l);
    }

    #[test]
    fn test_dict() {
        let b: Ben = "d3:foo3:bar6:hernani82ee".parse().unwrap();
        let mut m = Map::new();
        m.insert("foo".into(), Ben::String("bar".into()));
        m.insert("hernan".into(), Ben::Number(82));
        assert_eq!(b, Ben::Map(m));
    }
}
