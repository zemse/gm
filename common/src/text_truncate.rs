pub fn truncate_with_count(s: &str, limit: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= limit {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(limit).collect();
        let remaining = char_count - limit;
        format!("{truncated}...({remaining} more chars)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_with_count() {
        assert_eq!(truncate_with_count("hello", 10), "hello");
        assert_eq!(
            truncate_with_count("hello world", 5),
            "hello...(6 more chars)"
        );
        assert_eq!(truncate_with_count("hello world", 11), "hello world");
        assert_eq!(truncate_with_count("hello world", 0), "...(11 more chars)");
    }
}
