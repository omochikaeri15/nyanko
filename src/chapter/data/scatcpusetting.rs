use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::utils::csv;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum SCatCpuSettingError {
    EmptyFile,
}

impl fmt::Display for SCatCpuSettingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid Super Cat CPU setting data."
            ),
        }
    }
}

impl std::error::Error for SCatCpuSettingError {}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SCatCpuSetting {
    pub unknown_1: u32,
    pub super_cpu_daily_limit: u32,
    pub super_cpu_consume_amount: u32,
}

impl SCatCpuSetting {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, SCatCpuSettingError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<SCatCpuSetting, SCatCpuSettingError> {
    let file_content = csv::scrub(bytes);
    let separator_char = csv::detect_separator(&file_content);

    let mut setting = SCatCpuSetting::default();
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

        if let Some(val_string) = parts.first() {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                setting.unknown_1 = parsed_value;
            }
        }

        if let Some(val_string) = parts.get(1) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                setting.super_cpu_daily_limit = parsed_value;
            }
        }

        if let Some(val_string) = parts.get(2) {
            if let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                setting.super_cpu_consume_amount = parsed_value;
            }
        }
        break;
    }

    if !has_content {
        return Err(SCatCpuSettingError::EmptyFile);
    }

    Ok(setting)
}