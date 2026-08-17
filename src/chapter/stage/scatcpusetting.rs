//! Global configuration of the automatic play feature.
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::tools::file;

/// Represents errors that can occur during the parsing of automatic play settings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScatCpuSettingError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for ScatCpuSettingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid Super Cat CPU setting data."
            ),
        }
    }
}

impl std::error::Error for ScatCpuSettingError {}

/// The global configuration of the automatic play feature.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScatCpuSetting {
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_1: u32,
    /// The number of accelerated automatic play runs permitted each day.
    pub super_cpu_daily_limit: u32,
    /// The quantity of the consumable spent per accelerated run.
    pub super_cpu_consume_amount: u32,
}

impl ScatCpuSetting {
    /// Parses the automatic play settings file into its global configuration.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the automatic play settings file.
    ///
    /// # Returns
    /// A `Result` containing the parsed `ScatCpuSetting` on success, or a
    /// `ScatCpuSettingError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, ScatCpuSettingError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<ScatCpuSetting, ScatCpuSettingError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::detect_separator(&file_content);

    let mut setting = ScatCpuSetting::default();
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

        if let Some(val_string) = parts.first()
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                setting.unknown_1 = parsed_value;
            }

        if let Some(val_string) = parts.get(1)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                setting.super_cpu_daily_limit = parsed_value;
            }

        if let Some(val_string) = parts.get(2)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                setting.super_cpu_consume_amount = parsed_value;
            }
        break;
    }

    if !has_content {
        return Err(ScatCpuSettingError::EmptyFile);
    }

    Ok(setting)
}