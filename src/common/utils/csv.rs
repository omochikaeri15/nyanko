pub fn detect_separator(text: &str) -> char {
    if text.contains('|') {
        return '|';
    }

    text.chars()
        .find(|c| matches!(c, '\t' | ','))
        .unwrap_or(',')
}

pub fn scrub(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace(['\u{feff}', '\0'], "")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}