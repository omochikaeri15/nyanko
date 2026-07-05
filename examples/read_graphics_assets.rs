// examples/read_graphics_assets.rs

#[cfg(feature = "graphics")]
mod example {
    use std::fs;
    use std::path::Path;
    use nyanko::graphics::actor::{Unit, Animation, resolve_frame};

    pub fn run() {
        println!("Nyanko Graphics Pipeline Example\n");

        let asset_dir = Path::new("examples/assets");
        let png_path = asset_dir.join("DummyUnit.png");
        let imgcut_path = asset_dir.join("DummyUnit.imgcut");
        let mamodel_path = asset_dir.join("DummyUnit.mamodel");
        let maanim_path = asset_dir.join("DummyUnit.maanim");

        if !png_path.exists() || !mamodel_path.exists() {
            eprintln!("Missing dummy assets! Please run the generator first:");
            eprintln!("cargo run --example generate_dummy_graphics --features graphics");
            return;
        }

        // ==========================================
        // 1. Parsing the Core Unit Hierarchy
        // ==========================================
        println!("--- [ Loading Unit Assets ] ---");
        let png_bytes = fs::read(png_path).unwrap();
        let imgcut_bytes = fs::read(imgcut_path).unwrap();
        let mamodel_bytes = fs::read(mamodel_path).unwrap();

        let unit = Unit::parse(&png_bytes, &imgcut_bytes, &mamodel_bytes)
            .expect("Failed to parse unit hierarchy or texture atlas");

        println!("Successfully built Unit containing {} structural parts.", unit.model.parts.len());

        // ==========================================
        // 2. Loading the Animation Timeline
        // ==========================================
        let maanim_bytes = fs::read(maanim_path).unwrap();
        let animation = Animation::parse(&maanim_bytes)
            .expect("Failed to parse animation curves");

        println!("Successfully loaded animation (Max Frame: {})", animation.max_frame);

        // ==========================================
        // 3. Resolving Hardware-Agnostic FrameData
        // ==========================================
        println!("\n--- [ Resolving Frame Geometry ] ---");

        // Simulating the pipeline at exactly frame 5.0
        let target_frame = 5.0;
        let gpu_payload = resolve_frame(&unit, Some(&animation), target_frame);

        println!("GPU Draw Calls for Frame {}:", target_frame);
        for data in gpu_payload {
            println!("  -> Sprite Index : {}", data.sprite_index);
            println!("  -> Final Matrix : {:.2?}", data.final_matrix);
            println!("  -> Opacity      : {:.2}", data.opacity);
            // The vertices are formatted as a flat array of 12 floats (6 x/y pairs)
            println!("  -> Vertices (XY): {:.2?}", data.vertices);
        }
    }
}

fn main() {
    #[cfg(feature = "graphics")]
    example::run();

    #[cfg(not(feature = "graphics"))]
    eprintln!("Error: This example requires the 'graphics' feature.\nRun with: cargo run --example read_graphics_assets --features graphics");
}