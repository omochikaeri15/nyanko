use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::utils::csv;

#[derive(Debug)]
pub enum MapNameError {
    EmptyFile,
}

impl fmt::Display for MapNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid map name data."
            ),
        }
    }
}

impl std::error::Error for MapNameError {}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapName {
    pub names: HashMap<u32, String>,
}

impl MapName {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, MapNameError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<MapName, MapNameError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut names = HashMap::new();
    let mut has_content = false;

    for file_line in file_content.lines() {
        let mut clean_line = file_line;

        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if parts.len() < 2 {
            continue;
        }

        let Some(id_string) = parts.first() else { continue; };
        let Ok(map_id) = id_string.trim().parse::<u32>() else { continue; };

        let Some(name_string) = parts.get(1) else { continue; };
        let map_name = name_string.trim();

        if !map_name.is_empty() {
            names.insert(map_id, map_name.to_string());
        }
    }

    if !has_content {
        return Err(MapNameError::EmptyFile);
    }

    Ok(MapName { names })
}