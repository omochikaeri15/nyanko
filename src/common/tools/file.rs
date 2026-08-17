//! Sanitization and structural probing of raw delimited text files.
//!
//! The engine's data files vary in encoding, line ending, and delimiter between
//! regions and versions, so every parser normalizes its input through these
//! helpers before reading any columns.

/// Determines the delimiter a raw text payload uses.
///
/// # Arguments
/// * `text` - The sanitized text to probe.
///
/// # Returns
/// A `char` holding the detected delimiter, defaulting to a comma.
pub fn detect_separator(text: &str) -> char {
    if text.contains('|') {
        return '|';
    }

    text.chars()
        .find(|current| matches!(current, '\t' | ','))
        .unwrap_or(',')
}

/// Converts raw bytes to a string, dropping byte-order marks and null characters
/// and normalizing line endings.
///
/// # Arguments
/// * `bytes` - The raw bytes to sanitize.
///
/// # Returns
/// A `String` safe to traverse with the standard text utilities.
pub fn scrub(bytes: &[u8]) -> String {
    let raw_text = String::from_utf8_lossy(bytes);
    let mut clean_text = String::with_capacity(raw_text.len());
    let mut char_stream = raw_text.chars().peekable();

    while let Some(current) = char_stream.next() {
        if matches!(current, '\u{feff}' | '\0') {
            continue;
        }

        if current == '\r' {
            clean_text.push('\n');
            if let Some(&'\n') = char_stream.peek() {
                char_stream.next();
            }
            continue;
        }

        clean_text.push(current);
    }

    clean_text
}

/// How `<br>` tags are substituted when stripping HTML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakHandling {
    /// Replaces `<br>` with a single space, collapsing runs.
    Space,
    /// Replaces `<br>` with a newline character (`\n`).
    Newline,
    /// Completely deletes the `<br>` tag without injecting any substitute character.
    Delete,
}

/// Strips HTML tags from a string, handling `<br>` by the given strategy.
///
/// # Arguments
/// * `input` - The raw text to strip.
/// * `handling` - How `<br>` elements should be substituted.
///
/// # Returns
/// A `String` with tags removed and breaks substituted.
pub fn strip_html_tags(input: &str, handling: BreakHandling) -> String {
    let mut stripped = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find('<') {
        stripped.push_str(&rest[..start]);
        rest = &rest[start..];

        let Some(end) = rest.find('>') else {
            break;
        };

        let tag = &rest[..=end];
        rest = &rest[end + 1..];

        if tag.len() < 3 || !tag[..3].eq_ignore_ascii_case("<br") {
            continue;
        }

        match handling {
            BreakHandling::Space => {
                if !stripped.ends_with(' ') && !rest.starts_with(' ') {
                    stripped.push(' ');
                }
            }
            BreakHandling::Newline => stripped.push('\n'),
            BreakHandling::Delete => continue,
        }
    }

    stripped.push_str(rest);
    stripped
}

/// Finds a row by one column's value and returns another column from it.
///
/// # Arguments
/// * `data` - The raw bytes of the table to search.
/// * `key` - The value to match against `search_col`.
/// * `search_col` - The zero-indexed column to match on.
/// * `target_col` - The zero-indexed column to extract from the matched row.
///
/// # Returns
/// An `Option` containing the trimmed value, or `None` if no row matched.
pub fn lookup(
    data: &[u8],
    key: &str,
    search_col: usize,
    target_col: usize,
) -> Option<String> {
    let content = scrub(data);
    let separator = detect_separator(&content);

    for line in content.lines() {
        let Some(clean_line) = line.split("//").next() else {
            continue;
        };

        let trimmed_line = clean_line.trim();

        if trimmed_line.is_empty() {
            continue;
        }

        let Some(current_key) = trimmed_line.split(separator).nth(search_col) else {
            continue;
        };

        if current_key.trim() != key {
            continue;
        }

        let Some(target_value) = trimmed_line.split(separator).nth(target_col) else {
            continue;
        };

        return Some(target_value.trim().to_string());
    }

    None
}