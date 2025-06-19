use std::cmp::min;

pub fn split_string(s: &str, max_width: usize) -> Vec<&str> {
    let mut lines = vec![];

    let mut ptr = 0;
    let s_len = s.len();
    while ptr < s_len {
        let next = min(ptr + max_width, s_len);
        let s = s.get(ptr..next).expect("couldnt slice"); // can't go wrong
        lines.push(s);
        ptr = next;
    }

    if lines.is_empty() {
        lines.push("");
    }

    lines
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_string() {
        assert_eq!(
            split_string("hello what is up", 6),
            vec!["hello ", "what i", "s up"]
        );
    }
}
