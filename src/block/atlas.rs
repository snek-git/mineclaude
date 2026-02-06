use super::{BlockType, Face};

const ATLAS_TILES: f32 = 16.0;

/// Returns the texture tile index for a given block face.
/// The index maps to a position in the 16x16 texture atlas.
pub fn texture_index(block: BlockType, face: Face) -> u32 {
    match block {
        BlockType::Stone => 0,
        BlockType::Dirt => 1,
        BlockType::Grass => match face {
            Face::Top => 2,
            Face::Bottom => 1, // dirt
            _ => 3,            // grass side
        },
        BlockType::Cobblestone => 4,
        BlockType::Planks => 5,
        BlockType::Sand => 6,
        BlockType::Gravel => 7,
        BlockType::OakLog => match face {
            Face::Top | Face::Bottom => 8,
            _ => 9,
        },
        BlockType::OakLeaves => 10,
        BlockType::Glass => 11,
        BlockType::CoalOre => 12,
        BlockType::IronOre => 13,
        BlockType::GoldOre => 14,
        BlockType::DiamondOre => 15,
        BlockType::Bedrock => 16,
        BlockType::Water => 17,
        BlockType::CraftingTable => match face {
            Face::Top => 18,
            Face::Bottom => 5, // planks
            _ => 19,
        },
        BlockType::Furnace => match face {
            Face::Top | Face::Bottom => 20,
            Face::North => 21, // front face
            _ => 22,
        },
        BlockType::Snow => 23,
        BlockType::Clay => 24,
        BlockType::Sandstone => match face {
            Face::Top => 25,
            Face::Bottom => 26,
            _ => 27,
        },
        BlockType::BirchLog => match face {
            Face::Top | Face::Bottom => 28,
            _ => 29,
        },
        BlockType::BirchLeaves => 30,
        BlockType::Chest => match face {
            Face::Top => 33,
            Face::Bottom => 5, // planks
            Face::North => 31, // front (latch side)
            _ => 32,
        },
        BlockType::Bed => match face {
            Face::Top => 34,     // bed_head_top
            Face::Bottom => 5,   // planks underside
            _ => 35,             // bed_head_side for all sides
        },
        BlockType::DoorBottom | BlockType::DoorBottomOpen => 41, // door_wood_lower
        BlockType::DoorTop | BlockType::DoorTopOpen => 40, // door_wood_upper
        BlockType::OakSapling => 42,
        BlockType::BirchSapling => 43,
        BlockType::Farmland => match face {
            Face::Top => 44,
            _ => 1, // dirt sides
        },
        BlockType::WheatStage0 => 45,
        BlockType::WheatStage1 => 46,
        BlockType::WheatStage2 => 47,
        BlockType::WheatStage3 => 48,
        BlockType::Torch => 49,
        BlockType::TallGrass => 50,
        _ => 0, // Air — shouldn't be rendered
    }
}

/// Returns UV coordinates [u_min, v_min, u_max, v_max] for a tile index in the atlas.
pub fn tile_uvs(tile_index: u32) -> [f32; 4] {
    let col = (tile_index % ATLAS_TILES as u32) as f32;
    let row = (tile_index / ATLAS_TILES as u32) as f32;

    let u_min = col / ATLAS_TILES;
    let v_min = row / ATLAS_TILES;
    let u_max = u_min + 1.0 / ATLAS_TILES;
    let v_max = v_min + 1.0 / ATLAS_TILES;

    [u_min, v_min, u_max, v_max]
}

/// Returns the 4 UV corners for a face quad: [bottom-left, bottom-right, top-right, top-left]
pub fn face_uvs(block: BlockType, face: Face) -> [[f32; 2]; 4] {
    face_uvs_tiled(block, face, 1, 1)
}

/// Returns the 4 UV corners for a greedy-merged face quad that tiles the texture
/// `tile_w` times horizontally and `tile_h` times vertically.
/// The UVs span beyond the single tile boundary so the texture repeats across the merged quad.
pub fn face_uvs_tiled(block: BlockType, face: Face, tile_w: usize, tile_h: usize) -> [[f32; 2]; 4] {
    let [u_min, v_min, _u_max, _v_max] = tile_uvs(texture_index(block, face));
    let tile_size = 1.0 / ATLAS_TILES;
    let u_max = u_min + tile_size * tile_w as f32;
    let v_max = v_min + tile_size * tile_h as f32;
    [
        [u_min, v_max], // bottom-left
        [u_max, v_max], // bottom-right
        [u_max, v_min], // top-right
        [u_min, v_min], // top-left
    ]
}
