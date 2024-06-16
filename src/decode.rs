use anyhow::Result;

pub fn string(i: &str) -> Result<String> {
    let s = serde_bencode::from_str(i)?;
    let s = serde_json::Value::String(s);
    Ok(s.to_string())
}

pub fn integer(i: &str) -> Result<i64> {
    let i = serde_bencode::from_str(i)?;
    Ok(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string() {
        let s = string("9:hernan.rs").unwrap();
        assert_eq!(s, "hernan.rs")
    }

    #[test]
    fn test_i32() {
        let i = integer("i42e").unwrap();
        assert_eq!(i, 42);

        let i = integer("i-42e").unwrap();
        assert_eq!(i, -42);
    }
}
