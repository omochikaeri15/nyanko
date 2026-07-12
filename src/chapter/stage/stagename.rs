use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::csv;

#[derive(Debug)]
pub enum StageNameError {
    EmptyFile,
}

impl fmt::Display for StageNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid stage name data."
            ),
        }
    }
}

impl std::error::Error for StageNameError {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageNameEntry {
    pub names: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageName {
    pub entries: HashMap<u32, StageNameEntry>,
}

impl StageName {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, StageNameError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<StageName, StageNameError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut entries = HashMap::new();
    let mut has_content = false;
    
    for (line_index, file_line) in file_content.lines().enumerate() {
        let mut clean_line = file_line;
        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let names: Vec<String> = trimmed_line
            .split(separator_char)
            .map(|part| part.trim().to_string())
            .collect();

        entries.insert(
            line_index as u32,
            StageNameEntry { names },
        );
    }

    if !has_content {
        return Err(StageNameError::EmptyFile);
    }

    Ok(StageName { entries })
}