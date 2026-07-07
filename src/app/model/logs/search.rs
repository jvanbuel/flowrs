//! Vim-style search over log content.
//!
//! `/` opens the search bar and matches are highlighted while typing,
//! `Enter` confirms the query, and `n`/`N` cycle through the matches.

/// A single occurrence of the search query in the log content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchMatch {
    /// 0-based source line index in the log content.
    pub line: usize,
    /// Byte offset of the match start within its line.
    pub start: usize,
    /// Byte offset of the match end within its line.
    pub end: usize,
}

/// Search state of the log viewer.
#[derive(Default)]
pub enum Search {
    /// No search active, no highlights shown.
    #[default]
    Inactive,
    /// The user is typing in the search bar; matches update on every keystroke.
    Editing(SearchData),
    /// The query is confirmed; `n`/`N` navigate between matches.
    Applied(SearchData),
}

#[derive(Default)]
pub struct SearchData {
    pub query: String,
    /// All matches in the current log, ordered by position.
    pub matches: Vec<SearchMatch>,
    /// Index into `matches` of the current match.
    pub current: usize,
}

impl Search {
    pub fn is_editing(&self) -> bool {
        matches!(self, Search::Editing(_))
    }

    /// The search data while a query is being typed or is applied.
    pub fn data(&self) -> Option<&SearchData> {
        match self {
            Search::Inactive => None,
            Search::Editing(data) | Search::Applied(data) => Some(data),
        }
    }

    /// Recompute matches against `content`, keeping the current match index in bounds.
    pub fn refresh(&mut self, content: &str) {
        if let Search::Editing(data) | Search::Applied(data) = self {
            data.matches = find_matches(content, &data.query);
            data.current = data.current.min(data.matches.len().saturating_sub(1));
        }
    }
}

impl SearchData {
    pub fn current_match(&self) -> Option<SearchMatch> {
        self.matches.get(self.current).copied()
    }

    /// Move to the next/previous match, wrapping around. Returns the new
    /// current match, or `None` if there are no matches.
    pub fn cycle(&mut self, forward: bool) -> Option<SearchMatch> {
        let len = self.matches.len();
        if len == 0 {
            return None;
        }
        self.current = if forward {
            (self.current + 1) % len
        } else {
            (self.current + len - 1) % len
        };
        self.current_match()
    }
}

/// Find all non-overlapping occurrences of `query` in `content`, matching
/// ASCII case-insensitively (log content is virtually always ASCII).
pub fn find_matches(content: &str, query: &str) -> Vec<SearchMatch> {
    if query.is_empty() {
        return Vec::new();
    }
    let mut matches = Vec::new();
    for (line_idx, line) in content.lines().enumerate() {
        let mut start = 0;
        while start + query.len() <= line.len() {
            // `get` returns None when the end falls inside a multi-byte char
            match line.get(start..start + query.len()) {
                Some(candidate) if candidate.eq_ignore_ascii_case(query) => {
                    matches.push(SearchMatch {
                        line: line_idx,
                        start,
                        end: start + query.len(),
                    });
                    start += query.len();
                }
                _ => {
                    start += line[start..].chars().next().map_or(1, char::len_utf8);
                }
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(line: usize, start: usize, end: usize) -> SearchMatch {
        SearchMatch { line, start, end }
    }

    #[test]
    fn empty_query_matches_nothing() {
        assert!(find_matches("hello\nworld", "").is_empty());
    }

    #[test]
    fn no_match() {
        assert!(find_matches("hello\nworld", "foo").is_empty());
    }

    #[test]
    fn matches_are_per_occurrence_not_per_line() {
        assert_eq!(
            find_matches("error and error\nok\nerror", "error"),
            vec![m(0, 0, 5), m(0, 10, 15), m(2, 0, 5)]
        );
    }

    #[test]
    fn matching_is_ascii_case_insensitive() {
        assert_eq!(
            find_matches("ERROR here\nError there", "error"),
            vec![m(0, 0, 5), m(1, 0, 5)]
        );
    }

    #[test]
    fn matches_do_not_overlap() {
        assert_eq!(find_matches("aaa", "aa"), vec![m(0, 0, 2)]);
    }

    #[test]
    fn multibyte_content_yields_correct_byte_offsets() {
        // "é" is 2 bytes; the match must start at byte 3
        assert_eq!(find_matches("éé foo", "foo"), vec![m(0, 5, 8)]);
        // query overlapping a multi-byte boundary must not panic
        assert!(find_matches("ééé", "é\u{0065}").is_empty());
    }

    #[test]
    fn refresh_recomputes_and_clamps_current() {
        let mut search = Search::Applied(SearchData {
            query: "error".into(),
            matches: find_matches("error\nerror\nerror", "error"),
            current: 2,
        });
        search.refresh("error");
        let data = search.data().unwrap();
        assert_eq!(data.matches, vec![m(0, 0, 5)]);
        assert_eq!(data.current, 0);
    }

    #[test]
    fn cycle_wraps_in_both_directions() {
        let mut data = SearchData {
            query: "a".into(),
            matches: vec![m(0, 0, 1), m(1, 0, 1), m(2, 0, 1)],
            current: 0,
        };
        assert_eq!(data.cycle(true), Some(m(1, 0, 1)));
        assert_eq!(data.cycle(true), Some(m(2, 0, 1)));
        assert_eq!(data.cycle(true), Some(m(0, 0, 1)));
        assert_eq!(data.cycle(false), Some(m(2, 0, 1)));
    }

    #[test]
    fn cycle_with_no_matches_returns_none() {
        let mut data = SearchData::default();
        assert_eq!(data.cycle(true), None);
    }
}
