use std::path::{Path, PathBuf};

/// Calculates the chronological load weight of a pack file based on its filepath.
///
/// The Battle Cats uses a specific naming convention to determine which files overwrite
/// others in memory during extraction or loading. Higher calculated weights indicate
/// that the file should be loaded last.
///
/// # Arguments
/// * `path` - The full Path object of the file being evaluated.
/// * `temp_apk_dirs` - A list of temporary update/APK directories to boost score.
///
/// # Returns
/// A `u64` representing the sorting weight. Higher values take precedence in memory overlays.
pub fn calculate_weight(path: &Path, temp_apk_dirs: &[PathBuf]) -> u64 {
    let mut weight = 5_000;
    let mut found_version = false;

    let components: Vec<_> = path.components().map(|c| c.as_os_str().to_string_lossy().to_string()).collect();
    
    for comp in components.iter().rev() {
        let stem = comp.rsplit_once('.').map_or(comp.as_str(), |(name, _ext)| name);
        let parts: Vec<&str> = stem.split('_').collect();

        let len = parts.len();
        if len >= 3 {
            let maybe_minor = parts[len - 1].parse::<u64>();
            let maybe_major = parts[len - 2].parse::<u64>();

            if let (Ok(major), Ok(minor)) = (maybe_major, maybe_minor) {
                weight = 100_000_000 + (major * 100) + minor;
                found_version = true;
            } else if let (Ok(major), Ok(minor)) = (parts[1].parse::<u64>(), parts[2].parse::<u64>()) {
                weight = 100_000_000 + (major * 100) + minor;
                found_version = true;
            }
        }

        if found_version {
            break;
        }
    }

    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();

    if weight == 5_000 && file_stem.ends_with("Server") {
        let chars: Vec<char> = file_stem.chars().collect();
        weight = if chars.len() > 1 && chars[0].is_ascii_uppercase() && chars[1].is_ascii_uppercase() {
            20_000 + (chars[0] as u64)
        } else {
            10_000
        };
    }

    if weight == 5_000 && (file_stem == "DataLocal" || file_stem == "UpdateLocal" || file_stem.ends_with("Local")) {
        weight = 0;
    }

    if temp_apk_dirs.iter().any(|dir| path.starts_with(dir)) {
        weight += 500_000_000;
    }

    weight
}