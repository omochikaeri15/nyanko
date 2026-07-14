/// Dynamically determines the delimiter character utilized within a raw text payload.
///
/// # Arguments
/// * `text` - A string slice representing the raw data segment.
///
/// # Returns
/// A `char` representing the detected delimiter. Defaults to a comma (`,`) if no alternative is structurally identified.
pub fn detect_separator(text: &str) -> char {
    if text.contains('|') {
        return '|';
    }

    text.chars()
        .find(|current| matches!(current, '\t' | ','))
        .unwrap_or(',')
}

/// Sanitizes raw byte streams into a UTF-8 string format, stripping invalid bytes and normalizing line endings.
///
/// # Arguments
/// * `bytes` - A slice of raw bytes extracted directly from the filesystem or memory.
///
/// # Returns
/// A sanitized `String` guaranteed to be safely traversable by standard text utilities.
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

/// Dictates how `<br>` tags should be handled during the HTML stripping process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakHandling {
    /// Replaces `<br>` with a standard space (` `), ensuring safe horizontal spacing.
    Space,
    /// Replaces `<br>` with a newline character (`\n`).
    Newline,
    /// Completely deletes the `<br>` tag without injecting any substitute character.
    Delete,
}

/// Strips HTML tags from a string and processes `<br>` elements based on a specified handling strategy.
///
/// # Arguments
/// * `input` - A string slice representing the raw text containing HTML tags.
/// * `handling` - A `BreakHandling` enum variant dictating how `<br>` elements are processed.
///
/// # Returns
/// A clean `String` with HTML tags removed and `<br>` elements processed according to the chosen strategy.
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

/// Executes a stateless positional lookup within a raw CSV byte stream.
///
/// # Arguments
/// * `data` - A slice of raw bytes containing the target byte stream.
/// * `key` - The string identifier to locate within the target search axis.
/// * `search_col` - The zero-indexed column vector to scan for the provided identifier.
/// * `target_col` - The zero-indexed column vector utilized for extraction upon a match.
///
/// # Returns
/// An `Option<String>` containing the trimmed extracted value upon successful localization, or `None` if the target is not found.
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