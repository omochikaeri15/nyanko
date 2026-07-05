#[cfg(feature = "graphics")]
mod example {
    use std::fs;
    use std::path::Path;
    use image::{RgbaImage, Rgba};

    pub fn run() {
        println!("Nyanko Graphics Asset Generator");

        let asset_dir = Path::new("examples/assets");
        if let Err(e) = fs::create_dir_all(asset_dir) {
            eprintln!("Failed to create assets directory: {}", e);
            return;
        }

        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, Rgba([255, 255, 255, 255]));
        let png_path = asset_dir.join("DummyUnit.png");
        img.save(&png_path).expect("Failed to write PNG");
        println!("--> Generated synthetic texture atlas: {}", png_path.display());

        let dummy_imgcut = "1\n0,0,1,1,RootSprite";
        let imgcut_path = asset_dir.join("DummyUnit.imgcut");
        fs::write(&imgcut_path, dummy_imgcut).expect("Failed to write imgcut");
        println!("--> Generated synthetic sprite map: {}", imgcut_path.display());

        let dummy_mamodel = "3\n-1,0,0,0,0,0,0,0,1000,1000,0,1000,0,RootPart\n0,0,0,1,50,0,0,0,1000,1000,0,1000,0,ChildNode1\n1,0,0,2,50,0,0,0,1000,1000,0,1000,0,ChildNode2\n1000,3600,1000\n1\n0,0,0,0";
        let mamodel_path = asset_dir.join("DummyUnit.mamodel");
        fs::write(&mamodel_path, dummy_mamodel).expect("Failed to write mamodel");
        println!("--> Generated synthetic skeleton: {}", mamodel_path.display());

        let dummy_maanim = "[anim]\n1\n1\n0,4,1,0,10\n2\n0,0,0,0\n10,100,0,0";
        let maanim_path = asset_dir.join("DummyUnit.maanim");
        fs::write(&maanim_path, dummy_maanim).expect("Failed to write maanim");
        println!("--> Generated synthetic animation: {}", maanim_path.display());

        println!("\nSuccess! You can now run `cargo run --example read_graphics_assets --features graphics`");
    }
}

fn main() {
    #[cfg(feature = "graphics")]
    example::run();

    #[cfg(not(feature = "graphics"))]
    eprintln!("Error: This generator requires the 'graphics' feature.\nRun with: cargo run --example generate_dummy_graphics --features graphics");
}