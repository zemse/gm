use std::borrow::Cow;

use textwrap::{wrap, Options, WordSeparator};

pub fn text_wrap(text: &'_ str, max_width: u16) -> Vec<Cow<'_, str>> {
    wrap(
        text,
        Options::new(max_width as usize)
            .word_separator(WordSeparator::AsciiSpace)
            .break_words(true)
            .preserve_trailing_space(true),
    )
}

/// The wrapping done by textwrap crate removes new line characters, this function
/// reverse engineers this because given few lines we want to get the string slice.
/// This is used for getting the text by dragging on the rendered lines in TextScroll widget.
///
/// text: original text
/// lines: lines obtained by wrapping the original text
/// return: a vector of bools indicating whether each line ends with a new line character
pub fn has_new_line_char(text: &str, lines: &Vec<Cow<'_, str>>) -> Vec<bool> {
    let mut res = Vec::with_capacity(lines.len());
    let mut start_idx = 0;
    for line in lines {
        let end_idx = start_idx + line.len();
        if end_idx < text.len() && text.as_bytes()[end_idx] == b'\n' {
            res.push(true);
            start_idx = end_idx + 1; // skip the new line character
        } else {
            res.push(false);
            start_idx = end_idx;
        }
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_text_wrap_empty() {
        assert_eq!(text_wrap("", 6), vec![""]);
    }

    #[test]
    fn test_text_wrap_1() {
        assert_eq!(
            text_wrap("hello what is up", 6),
            vec!["hello ", "what ", "is up"]
        );
    }

    #[test]
    fn test_text_wrap_2() {
        assert_eq!(
            text_wrap("hellohello what is up", 6),
            vec!["helloh", "ello ", "what ", "is up"]
        );
    }

    #[test]
    fn test_text_wrap_3() {
        // new line characters disappear
        assert_eq!(
            text_wrap("hellohello wh\n\nat is up\t", 6),
            vec!["helloh", "ello ", "wh", "", "at is ", "up\t"]
        );
    }

    #[test]
    fn test_new_line_chars_1() {
        let text = "hello\n what is up";
        let lines = text_wrap(text, 6);
        assert_eq!(lines, vec!["hello", " what ", "is up"]);
        assert_eq!(has_new_line_char(text, &lines), vec![true, false, false]);
    }
}
