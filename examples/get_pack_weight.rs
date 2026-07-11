#[cfg(feature = "pack")]
mod example {
    use std::path::{Path, PathBuf};
    use nyanko::pack::chronology::calculate_weight;

    pub fn run() {
        println!("Nyanko Chronology API Example");

        // The 'temp_apk_dirs' parameter provides a massive weight boost to active extractions.
        // Paths residing inside these directories will confidently override standard local files.
        let temp_apk_dirs = vec![PathBuf::from("/tmp/nyanko_apk_extract")];

        // 1. Standard Local Packs
        // Base engine files usually carry a weight of 0 to allow for easy overriding in memory.
        let data_local = Path::new("DataLocal.pack");
        println!(
            "Weight for {}: {}",
            data_local.display(),
            calculate_weight(data_local, &temp_apk_dirs)
        );

        // 2. Server Packs
        // Files starting with capitalization calculate their weight based on ASCII values.
        // E.g., 'E' in ENServer calculates specific hierarchy weight.
        let server_pack = Path::new("ENServer.pack");
        println!(
            "Weight for {}: {}",
            server_pack.display(),
            calculate_weight(server_pack, &temp_apk_dirs)
        );

        // 3. Versioned Game Updates
        // Parses the major and minor version numbers directly from the filename to calculate hierarchy.
        let versioned_pack = Path::new("game_14_2.pack");
        println!(
            "Weight for {}: {}",
            versioned_pack.display(),
            calculate_weight(versioned_pack, &temp_apk_dirs)
        );

        // 4. Custom Modding Assets
        // User assets stored in the strict 'patch' directory format maintain a standard baseline weight.
        let patch_pack = Path::new("patch/custom_ui.pack");
        println!(
            "Weight for {}: {}",
            patch_pack.display(),
            calculate_weight(patch_pack, &temp_apk_dirs)
        );

        // 5. Server Timestamp Identifiers
        // Simulates evaluating a server timestamp file residing inside an active APK extraction.
        // This triggers the temp_apk_dirs overlay boost.
        let timestamp_pack = Path::new("/tmp/nyanko_apk_extract/15040000.pack");
        println!(
            "Weight for {}: {}",
            timestamp_pack.display(),
            calculate_weight(timestamp_pack, &temp_apk_dirs)
        );
    }
}

fn main() {
    #[cfg(feature = "pack")]
    example::run();

    #[cfg(not(feature = "pack"))]
    eprintln!("Error: This example requires the 'pack' feature.\nRun with: cargo run --example pack_chronology --features pack");
}