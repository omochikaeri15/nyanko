#[cfg(feature = "pack")]
mod example {
    use std::fs;
    use std::path::Path;
    use nyanko::pack::cryptology::{Keys, decrypt_list, decrypt_chunk};
    use nyanko::common::Region;

    /// The 16-byte hex-encoded key for regional asset decryption.
    const TARGET_KEY_HEX: &str = "0123456789abcdef0123456789abcdef";
    /// The 16-byte hex-encoded initialization vector.
    const TARGET_IV_HEX:  &str = "fedcba9876543210fedcba9876543210";
    /// The target region for the decryption routine.
    const TARGET_REGION: Region = Region::En;

    pub fn run() {
        println!("Nyanko Cryptology File Reader Example\n");

        let asset_dir = Path::new("examples/assets");
        let list_path = asset_dir.join("DummyLocal.list");
        let pack_path = asset_dir.join("DummyLocal.pack");

        if !list_path.exists() || !pack_path.exists() {
            eprintln!("Missing assets. Ensure the target files exist in examples/assets/.");
            return;
        }

        println!("--- [ Reading Manifest List ] ---");

        let encrypted_list_bytes = fs::read(&list_path).expect("Failed to read list file");

        let manifest_text = match decrypt_list(&encrypted_list_bytes) {
            Ok(manifest) => {
                println!("List decrypted successfully.\n");
                println!("--- [ Manifest Preview ] ---");

                // Previews the first 5 lines to demonstrate standard PONOS index formatting.
                for (index, line) in manifest.lines().enumerate().take(5) {
                    println!("Line {}: {}", index + 1, line);
                }
                println!("...\n");

                manifest
            },
            Err(e) => {
                eprintln!("List decryption failed: {}", e);
                return;
            }
        };

        // Parses the manifest to identify the byte boundaries of the target file.
        // Format requirement: Filename,Offset,Size
        let mut target_filename = String::new();
        let mut target_offset: usize = 0;
        let mut target_size: usize = 0;

        for line in manifest_text.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 && parts[0].ends_with(".csv") {
                target_filename = parts[0].to_string();
                target_offset = parts[1].parse().unwrap_or(0);
                target_size = parts[2].parse().unwrap_or(0);
                break;
            }
        }

        if target_filename.is_empty() {
            eprintln!("No CSV files found in the manifest.");
            return;
        }

        println!("Targeting: {} (Offset: {}, Size: {})", target_filename, target_offset, target_size);

        println!("\n--- [ Processing Regional Pack ] ---");

        let key_tuples = vec![(TARGET_REGION, TARGET_KEY_HEX, TARGET_IV_HEX)];

        let keys = match Keys::parse(&key_tuples) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("Key parsing failed: {}", e);
                return;
            }
        };

        let full_pack_bytes = fs::read(&pack_path).expect("Failed to read pack file");

        let end_bound = target_offset + target_size;
        if end_bound > full_pack_bytes.len() {
            eprintln!("Error: Manifest offset/size bounds exceed actual file length.");
            return;
        }

        // Slices the specific encrypted payload from the master archive using the manifest boundaries.
        let chunk_bytes = &full_pack_bytes[target_offset..end_bound];

        let (decrypted_data, region_used) = decrypt_chunk(chunk_bytes, &target_filename, &keys);

        if let Some(reg) = region_used {
            println!("Success! Chunk decrypted using regional CBC key: {:?}", reg);
            println!("\n--- [ Payload Preview ] ---");

            let payload_text = String::from_utf8_lossy(&decrypted_data);

            for (index, line) in payload_text.lines().enumerate().take(3) {
                println!("Line {}: {}", index + 1, line);
            }
        } else {
            println!("Chunk processed via server ECB fallback or passed raw. Byte len: {}", decrypted_data.len());
        }
    }
}

fn main() {
    #[cfg(feature = "pack")]
    example::run();

    #[cfg(not(feature = "pack"))]
    eprintln!("Error: This example requires the 'pack' feature.\nRun with: cargo run --example read_pack_assets --features pack");
}