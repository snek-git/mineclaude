use image::{GenericImageView, RgbaImage, Rgba};
use std::path::Path;

const ATLAS_SIZE: u32 = 256;
const TILE_SIZE: u32 = 16;
const TILES_PER_ROW: u32 = ATLAS_SIZE / TILE_SIZE;

/// Maps tile index → source texture filename.
/// Order must match atlas.rs texture_index().
const TILE_SOURCES: &[(u32, &str)] = &[
    (0, "stone.png"),
    (1, "dirt.png"),
    (2, "grass_top.png"),
    (3, "grass_side.png"),
    (4, "cobblestone.png"),
    (5, "planks.png"),
    (6, "sand.png"),
    (7, "gravel.png"),
    (8, "oak_log_top.png"),
    (9, "oak_log_side.png"),
    (10, "oak_leaves.png"),
    (11, "glass.png"),
    (12, "coal_ore.png"),
    (13, "iron_ore.png"),
    (14, "gold_ore.png"),
    (15, "diamond_ore.png"),
    (16, "bedrock.png"),
    (17, "water.png"),           // animated spritesheet — we take the first 16x16 frame
    (18, "crafting_table_top.png"),
    (19, "crafting_table_side.png"),
    (20, "furnace_top.png"),
    (21, "furnace_front.png"),
    (22, "furnace_side.png"),
    (23, "snow.png"),
    (24, "clay.png"),
    (25, "sandstone_top.png"),
    (26, "sandstone_bottom.png"),
    (27, "sandstone_side.png"),
    (28, "birch_log_top.png"),
    (29, "birch_log_side.png"),
    (30, "birch_leaves.png"),
    (31, "chest_front.png"),
    (32, "chest_side.png"),
    (33, "chest_top.png"),
    (34, "bed_head_top.png"),
    (35, "bed_head_side.png"),
    (36, "bed_head_end.png"),
    (37, "bed_feet_top.png"),
    (38, "bed_feet_side.png"),
    (39, "bed_feet_end.png"),
    (40, "door_wood_upper.png"),
    (41, "door_wood_lower.png"),
    (42, "sapling_oak.png"),
    (43, "sapling_birch.png"),
    (44, "farmland_top.png"),
    (45, "wheat_stage_0.png"),
    (46, "wheat_stage_1.png"),
    (47, "wheat_stage_2.png"),
    (48, "wheat_stage_3.png"),
    (49, "torch_on.png"),
    (50, "tallgrass.png"),
];

/// Biome tint colors for grayscale textures (plains biome).
/// Minecraft ships these textures as grayscale and tints them per-biome at runtime.
fn biome_tint(tile_index: u32) -> Option<(u8, u8, u8)> {
    match tile_index {
        2 => Some((124, 189, 107)),  // grass_top — plains green
        3 => Some((124, 189, 107)),  // grass_side — plains green
        10 => Some((119, 171, 47)), // oak_leaves — plains green
        17 => Some((63, 118, 228)), // water — default blue
        30 => Some((128, 167, 85)), // birch_leaves — birch green
        50 => Some((124, 189, 107)), // tallgrass — plains green
        _ => None,
    }
}

fn tint_pixel(pixel: Rgba<u8>, tint: (u8, u8, u8)) -> Rgba<u8> {
    Rgba([
        ((pixel[0] as u16 * tint.0 as u16) / 255) as u8,
        ((pixel[1] as u16 * tint.1 as u16) / 255) as u8,
        ((pixel[2] as u16 * tint.2 as u16) / 255) as u8,
        pixel[3],
    ])
}

fn generate_crack_textures() {
    let out_dir = Path::new("assets/textures");
    for stage in 0..10u32 {
        let path = out_dir.join(format!("destroy_stage_{}.png", stage));
        if path.exists() {
            continue;
        }
        let mut img = RgbaImage::new(16, 16);
        let density = (stage + 1) as f32 / 10.0;
        for y in 0..16u32 {
            for x in 0..16u32 {
                // Deterministic hash for consistent pattern
                let mut h = x.wrapping_mul(374761393)
                    .wrapping_add(y.wrapping_mul(668265263))
                    .wrapping_add(stage.wrapping_mul(2147483647));
                h = (h ^ (h >> 13)).wrapping_mul(1274126177);
                h = h ^ (h >> 16);
                let hash_val = (h & 0xFF) as f32 / 255.0;
                if hash_val < density {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 160]));
                } else {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }
        img.save(&path).expect("Failed to save crack texture");
    }
}

fn main() {
    generate_crack_textures();

    let out_path = Path::new("assets/textures/atlas.png");
    let blocks_dir = Path::new("assets/textures/blocks");

    std::fs::create_dir_all("assets/textures").expect("Failed to create textures directory");

    let mut atlas = RgbaImage::new(ATLAS_SIZE, ATLAS_SIZE);

    // Fill with magenta (missing texture indicator)
    for pixel in atlas.pixels_mut() {
        *pixel = Rgba([255, 0, 255, 255]);
    }

    for &(index, filename) in TILE_SOURCES {
        let src_path = blocks_dir.join(filename);
        if !src_path.exists() {
            eprintln!("WARNING: Missing texture {}", src_path.display());
            continue;
        }

        let src = image::open(&src_path)
            .unwrap_or_else(|e| panic!("Failed to open {}: {}", src_path.display(), e));

        let tint = biome_tint(index);

        // Only use the first 16x16 pixels (handles animated spritesheets like water)
        let col = index % TILES_PER_ROW;
        let row = index / TILES_PER_ROW;
        let x0 = col * TILE_SIZE;
        let y0 = row * TILE_SIZE;

        for dy in 0..TILE_SIZE {
            for dx in 0..TILE_SIZE {
                if dx < src.width() && dy < src.height() {
                    let mut pixel = src.get_pixel(dx, dy);
                    if let Some(t) = tint {
                        pixel = tint_pixel(pixel, t);
                    }
                    atlas.put_pixel(x0 + dx, y0 + dy, pixel);
                }
            }
        }
    }

    atlas.save(out_path).expect("Failed to save atlas.png");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/textures/blocks");
}
