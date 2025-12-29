use std::{borrow::Cow, cmp::Ordering};

use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use crate::text_wrap::text_wrap;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct WrappedSegment {
    pub idx: usize,
    pub kind: TokenKind,
    pub start_line: usize,
    pub start_char_idx: usize,
    pub end_line: usize,
    pub end_char_idx: usize,
}

/// Wrap the input string into lines of maximum `max_width` characters, and also find
/// where the URLs and hex strings are located.
pub fn segmented_wrap(str: &str, max_width: u16) -> (Vec<Cow<'_, str>>, Vec<WrappedSegment>) {
    let lines = text_wrap(str, max_width);
    let mut segments = scan_tokens(str);

    // text wrap removes new line characters, so we need to patch segment positions
    for (idx, _) in str.match_indices('\n') {
        for segment in &mut segments {
            if segment.start > idx {
                segment.start -= 1;
            }
            if segment.end > idx {
                segment.end -= 1;
            }
        }
    }

    let mut wrapped_segments = Vec::new();

    for (idx, segment) in segments.into_iter().enumerate() {
        let mut start_line = 0;
        let mut start_char_idx = segment.start;
        while start_line < lines.len() && start_char_idx >= lines[start_line].len() {
            start_char_idx -= lines[start_line].len();
            start_line += 1;
        }

        let mut end_line = start_line;
        let mut end_char_idx = segment.end - segment.start + start_char_idx;
        while end_line < lines.len() && end_char_idx > lines[end_line].len() {
            end_char_idx -= lines[end_line].len();
            end_line += 1;
        }

        wrapped_segments.push(WrappedSegment {
            idx,
            kind: segment.kind,
            start_line,
            start_char_idx,
            end_line,
            end_char_idx,
        });
    }

    wrapped_segments.sort_by(|a, b| {
        if matches!(a.kind, TokenKind::Hex(_)) == matches!(b.kind, TokenKind::Hex(_)) {
            Ordering::Equal
        } else if matches!(a.kind, TokenKind::Hex(_)) {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    (lines, wrapped_segments)
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum TokenKind {
    Url(Url),
    Hex(String),
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Segment {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

// Match http and https URLs
static URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?xi)\b(?:https?)://[^\s<>"'(){}\[\]]+"#).unwrap());

// Match hex strings starting with 0x followed by one or more hex digits
static HEX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?xi)\b0x[0-9a-fA-F]+\b"#).unwrap());

pub fn scan_tokens(s: &str) -> Vec<Segment> {
    let mut out = Vec::new();

    for m in URL_RE.find_iter(s) {
        let url = m.as_str().to_string().parse::<Url>();
        if let Ok(url) = url {
            out.push(Segment {
                kind: TokenKind::Url(url),
                start: m.start(),
                end: m.end(),
            });
        }
    }

    for m in HEX_RE.find_iter(s) {
        out.push(Segment {
            kind: TokenKind::Hex(m.as_str().to_string()),
            start: m.start(),
            end: m.end(),
        });
    }

    out.sort_by_key(|sp| sp.start);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_and_hex_1() {
        assert_eq!(
            scan_tokens(
                "see https://example.com/a?b=c and 0xDEAD plus deadbeef and ftp://host/file"
            ),
            vec![
                Segment {
                    kind: TokenKind::Url(Url::parse("https://example.com/a?b=c").unwrap()),
                    start: 4,
                    end: 29,
                },
                Segment {
                    kind: TokenKind::Hex("0xDEAD".to_string()),
                    start: 34,
                    end: 40,
                },
            ]
        );
    }

    #[test]
    fn url_and_hex_2() {
        assert_eq!(
            scan_tokens("check http://github.com/ and the hash is 0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d"),
            vec![
                Segment {
                    kind: TokenKind::Url(Url::parse("http://github.com/").unwrap()),
                    start: 6,
                    end: 24,
                },
                Segment {
                    kind: TokenKind::Hex("0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d".to_string()),
                    start: 41,
                    end: 107,
                },
            ]
        );
    }

    #[test]
    fn url_and_hex_3() {
        assert_eq!(
            scan_tokens("https://etherscan.io/tx/0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d"),
            vec![
                Segment {
                    kind: TokenKind::Url(Url::parse("https://etherscan.io/tx/0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d").unwrap()),
                    start: 0,
                    end: 90,
                },
                Segment {
                    kind: TokenKind::Hex("0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d".to_string()),
                    start: 24,
                    end: 90,
                },
            ]
        );
    }

    #[test]
    fn url_solo_1() {
        assert_eq!(
            scan_tokens("http://github.com/"),
            vec![Segment {
                kind: TokenKind::Url(Url::parse("http://github.com/").unwrap()),
                start: 0,
                end: 18,
            }]
        );
    }

    #[test]
    fn url_solo_2() {
        assert_eq!(
            scan_tokens(" http://github.com/ "),
            vec![Segment {
                kind: TokenKind::Url(Url::parse("http://github.com/").unwrap()),
                start: 1,
                end: 19,
            }]
        );
    }

    #[test]
    fn url_solo_3() {
        assert_eq!(scan_tokens(" http:// ",), vec![]);
    }

    #[test]
    fn url_solo_4() {
        assert_eq!(
            scan_tokens(" http://a ",),
            vec![Segment {
                kind: TokenKind::Url(Url::parse("http://a").unwrap()),
                start: 1,
                end: 9,
            }]
        );
    }

    #[test]
    fn hex_solo_1() {
        assert_eq!(
            scan_tokens("0x1234abcd",),
            vec![Segment {
                kind: TokenKind::Hex("0x1234abcd".to_string()),
                start: 0,
                end: 10,
            }]
        );
    }

    #[test]
    fn hex_solo_2() {
        assert_eq!(
            scan_tokens("0x1234abcd/",),
            vec![Segment {
                kind: TokenKind::Hex("0x1234abcd".to_string()),
                start: 0,
                end: 10,
            }]
        );
    }

    #[test]
    fn hex_solo_3() {
        assert_eq!(scan_tokens("z0x1234abcd"), vec![]);
    }

    #[test]
    fn segmented_wrap_1() {
        let (lines, segments) = segmented_wrap("hello https://letsgo", 7);

        assert_eq!(lines, vec!["hello ", "https:/", "/letsgo"]);
        assert_eq!(
            segments,
            vec![WrappedSegment {
                idx: 0,
                kind: TokenKind::Url(Url::parse("https://letsgo").unwrap()),
                start_line: 1,
                start_char_idx: 0,
                end_line: 2,
                end_char_idx: 7
            }]
        );
    }

    #[test]
    fn segmented_wrap_2() {
        let (lines, segments) = segmented_wrap("hello\nhttps://letsgo/whatsgoingon/here?", 11);

        assert_eq!(
            lines,
            vec!["hello", "https://let", "sgo/whatsgo", "ingon/here?"]
        );
        assert_eq!(
            segments,
            vec![WrappedSegment {
                idx: 0,
                kind: TokenKind::Url(Url::parse("https://letsgo/whatsgoingon/here?").unwrap()),
                start_line: 1,
                start_char_idx: 0,
                end_line: 3,
                end_char_idx: 11
            }]
        );
    }

    #[test]
    fn segmented_wrap_3() {
        let (lines, segments) = segmented_wrap("i found a url https://etherscan.io/tx/0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d on the internet", 40);

        assert_eq!(
            lines,
            vec![
                "i found a url ",
                "https://etherscan.io/tx/0x6d958274cf0278",
                "2d4dd261e920460d4d6620d49b03a07d13a4250a",
                "d694c9342d on the internet"
            ]
        );
        assert_eq!(
            segments,
            vec![
                WrappedSegment {
                    idx: 0,
                    kind: TokenKind::Url(Url::parse("https://etherscan.io/tx/0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d").unwrap()),
                    start_line: 1,
                    start_char_idx: 0,
                    end_line: 3,
                    end_char_idx: 10
                },
                WrappedSegment {
                    idx: 1,
                    kind: TokenKind::Hex("0x6d958274cf02782d4dd261e920460d4d6620d49b03a07d13a4250ad694c9342d".to_string()),
                    start_line: 1,
                    start_char_idx: 24,
                    end_line: 3,
                    end_char_idx: 10
                }
            ]
        );
    }
}
