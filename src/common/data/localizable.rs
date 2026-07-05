use std::collections::HashMap;
use std::fmt;

use crate::common::utils::csv;

/// Represents an error encountered during the localization dictionary build phase.
#[derive(Debug)]
pub enum LocalizableError {
    /// Indicates that the provided byte payload yielded no actionable string pairs.
    EmptyFile,
}

impl fmt::Display for LocalizableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalizableError::EmptyFile => write!(f, "The provided file bytes contained no valid localization data."),
        }
    }
}

impl std::error::Error for LocalizableError {}

/// Provides an optimized, zero-allocation dictionary for translating internal engine keys
/// into user-facing localized text.
#[derive(Debug, Clone, Default)]
pub struct Localizable {
    map: HashMap<String, String>,
}

impl Localizable {
    /// Parses raw TSV byte data into an indexed dictionary.
    ///
    /// # Arguments
    /// * `bytes` - A polymorphic byte slice reference representing the localization payload.
    ///
    /// # Returns
    /// A populated `Localizable` instance, or a `LocalizableError` if the payload is empty.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, LocalizableError> {
        parse_inner(bytes.as_ref())
    }

    /// Executes a zero-allocation query against the internal localization dictionary.
    ///
    /// # Arguments
    /// * `key` - The string identifier to locate within the dictionary.
    ///
    /// # Returns
    /// An `Option<&str>` containing a reference to the localized string, or `None` if the key is unmatched.
    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<Localizable, LocalizableError> {
    let content = csv::scrub(bytes);
    let estimated_entries = bytes.len() / 50;
    let mut map = HashMap::with_capacity(estimated_entries);

    for line in content.lines() {
        let clean_line = line.split("//").next().unwrap_or("").trim();

        if clean_line.is_empty() {
            continue;
        }

        let Some(tab_index) = clean_line.find('\t') else {
            continue;
        };

        let key = clean_line[..tab_index].trim().to_string();
        let value = clean_line[tab_index..].trim().to_string();

        map.insert(key, value);
    }

    if map.is_empty() {
        return Err(LocalizableError::EmptyFile);
    }

    Ok(Localizable { map })
}