#[cfg(feature = "pack")]
mod example {
    use nyanko::pack::cryptology::{encrypt_list, encrypt_chunk, PackType};
    use std::fs;
    use std::path::Path;

    pub fn run() {
        println!("Nyanko Asset Generator");

        let asset_dir = Path::new("examples/assets");
        if let Err(e) = fs::create_dir_all(asset_dir) {
            eprintln!("Failed to create assets directory: {}", e);
            return;
        }

        let key_bytes: [u8; 16] = hex::decode("0123456789abcdef0123456789abcdef")
            .unwrap().try_into().unwrap();
        let iv_bytes: [u8; 16] = hex::decode("fedcba9876543210fedcba9876543210")
            .unwrap().try_into().unwrap();

        let dummy_csv_payload = "FORM,HP,ATK,RANGE\n1,100,50,250\n2,120,80,250";

        let encrypted_csv_bytes = match encrypt_chunk(
            dummy_csv_payload.as_bytes(),
            PackType::Standard,
            Some(&key_bytes),
            Some(&iv_bytes),
        ) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to encrypt pack: {}", e);
                return;
            }
        };

        let mut archive_bytes = Vec::new();

        let dummy_png_size = 1500;
        archive_bytes.extend(vec![0u8; dummy_png_size]);

        let csv_offset = archive_bytes.len();
        let csv_size = encrypted_csv_bytes.len();
        archive_bytes.extend(&encrypted_csv_bytes);

        let stage_offset = archive_bytes.len();
        let dummy_stage_size = 2400;
        archive_bytes.extend(vec![0u8; dummy_stage_size]);

        let pack_path = asset_dir.join("DummyLocal.pack");
        fs::write(&pack_path, &archive_bytes).expect("Failed to write DummyLocal.pack");

        println!("--> Generated synthetic regional pack: {}", pack_path.display());

        let dummy_manifest = format!(
            "ui_001.png,0,{}\nunit_01.csv,{},{}\nstage_data.csv,{},{}",
            dummy_png_size,
            csv_offset,
            csv_size,
            stage_offset,
            dummy_stage_size
        );

        match encrypt_list(&dummy_manifest) {
            Ok(bytes) => {
                let path = asset_dir.join("DummyLocal.list");
                fs::write(&path, bytes).expect("Failed to write DummyLocal.list");
                println!("--> Generated synthetic manifest: {}", path.display());
            }
            Err(e) => eprintln!("Failed to encrypt manifest: {}", e),
        }

        println!("\nSuccess! You can now run `cargo run --example read_pack_assets --features pack`");
    }
}

fn main() {
    #[cfg(feature = "pack")]
    example::run();

    #[cfg(not(feature = "pack"))]
    eprintln!("Error: This generator requires the 'pack' feature.\nRun with: cargo run --example generate_dummy_assets --features pack");
}