use bevy::prelude::*;
use bevy::image::Image;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::block::BlockType;
use crate::player::Player;
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::world::generation::generate_chunk;

/// Size of the debug map in world blocks (and pixels).
const MAP_SIZE: usize = 128;

/// Resource tracking debug map state.
#[derive(Resource)]
pub struct DebugMapState {
    pub open: bool,
    pub y_level: i32,
    pub mode: DebugMapMode,
    pub dirty: bool,
    /// The center of the map in world block coordinates when last generated.
    pub center_x: i32,
    pub center_z: i32,
    /// Cached generated chunk data for the viewed region.
    /// Indexed as [cy][cz][cx] where cy is the vertical chunk index.
    chunks: Vec<Vec<Vec<Chunk>>>,
    /// Range of chunk-Y indices stored. chunks[0] corresponds to chunk_y_min.
    chunk_y_min: i32,
    chunk_y_max: i32,
    /// Chunk origin (world chunk coords) for XZ.
    chunk_x_origin: i32,
    chunk_z_origin: i32,
    /// Heightmap cache: [z][x] in block coords relative to map origin.
    heightmap: Vec<Vec<i32>>,
}

impl Default for DebugMapState {
    fn default() -> Self {
        Self {
            open: false,
            y_level: 64,
            mode: DebugMapMode::Block,
            dirty: true,
            center_x: 0,
            center_z: 0,
            chunks: Vec::new(),
            chunk_y_min: 0,
            chunk_y_max: 0,
            chunk_x_origin: 0,
            chunk_z_origin: 0,
            heightmap: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DebugMapMode {
    Block,
    Heightmap,
    Cave,
    Biome,
}

impl DebugMapMode {
    fn next(self) -> Self {
        match self {
            Self::Block => Self::Heightmap,
            Self::Heightmap => Self::Cave,
            Self::Cave => Self::Biome,
            Self::Biome => Self::Block,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Block => "Block",
            Self::Heightmap => "Heightmap",
            Self::Cave => "Cave",
            Self::Biome => "Biome",
        }
    }
}

/// Marker for the debug map UI root node.
#[derive(Component)]
pub(crate) struct DebugMapRoot;

/// Marker for the debug map image node.
#[derive(Component)]
pub(crate) struct DebugMapImage;

/// Marker for the debug map info text.
#[derive(Component)]
pub(crate) struct DebugMapText;

/// Handle to the debug map image asset.
#[derive(Resource)]
pub(crate) struct DebugMapImageHandle(Handle<Image>);

fn block_color(block: BlockType) -> [u8; 4] {
    match block {
        BlockType::Air => [0, 0, 0, 255],
        BlockType::Stone => [128, 128, 128, 255],
        BlockType::Dirt => [139, 90, 43, 255],
        BlockType::Grass => [76, 153, 0, 255],
        BlockType::Cobblestone => [120, 120, 120, 255],
        BlockType::Planks => [180, 140, 80, 255],
        BlockType::Sand => [237, 201, 175, 255],
        BlockType::Gravel => [150, 140, 130, 255],
        BlockType::OakLog | BlockType::BirchLog => [101, 67, 33, 255],
        BlockType::OakLeaves | BlockType::BirchLeaves => [34, 139, 34, 255],
        BlockType::Glass => [200, 220, 255, 255],
        BlockType::CoalOre => [64, 64, 64, 255],
        BlockType::IronOre => [180, 140, 100, 255],
        BlockType::GoldOre => [255, 215, 0, 255],
        BlockType::DiamondOre => [0, 255, 255, 255],
        BlockType::Bedrock => [32, 32, 32, 255],
        BlockType::Water => [30, 80, 200, 255],
        BlockType::CraftingTable => [160, 120, 60, 255],
        BlockType::Furnace => [100, 100, 100, 255],
        BlockType::Torch => [255, 200, 50, 255],
        BlockType::Snow => [240, 240, 255, 255],
        BlockType::Clay => [160, 165, 175, 255],
        BlockType::Sandstone => [220, 200, 150, 255],
        BlockType::TallGrass => [50, 130, 50, 255],
        BlockType::Chest => [160, 120, 50, 255],
        BlockType::Bed => [180, 50, 50, 255],
        BlockType::DoorBottom | BlockType::DoorTop
        | BlockType::DoorBottomOpen | BlockType::DoorTopOpen => [160, 120, 60, 255],
        BlockType::OakSapling => [34, 139, 34, 255],
        BlockType::BirchSapling => [50, 160, 50, 255],
        BlockType::Farmland => [100, 65, 25, 255],
        BlockType::WheatStage0 => [80, 140, 40, 255],
        BlockType::WheatStage1 => [100, 155, 40, 255],
        BlockType::WheatStage2 => [155, 165, 40, 255],
        BlockType::WheatStage3 => [200, 190, 50, 255],
    }
}

/// Toggle the debug map open/closed with F4.
pub fn toggle_debug_map(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugMapState>,
) {
    if keys.just_pressed(KeyCode::F4) {
        state.open = !state.open;
        if state.open {
            state.dirty = true;
        }
    }
}

/// Handle Y level and mode changes.
pub fn debug_map_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugMapState>,
) {
    if !state.open {
        return;
    }

    if keys.just_pressed(KeyCode::ArrowUp) {
        state.y_level = (state.y_level + 1).min(255);
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        state.y_level = (state.y_level - 1).max(0);
        state.dirty = true;
    }
    if keys.just_pressed(KeyCode::Tab) {
        state.mode = state.mode.next();
        state.dirty = true;
    }
}

/// Spawn the debug map UI when it becomes open.
pub fn spawn_debug_map_ui(
    mut commands: Commands,
    state: Res<DebugMapState>,
    mut images: ResMut<Assets<Image>>,
    existing: Query<Entity, With<DebugMapRoot>>,
    _handle_res: Option<Res<DebugMapImageHandle>>,
) {
    if !state.open {
        return;
    }
    // Already spawned
    if !existing.is_empty() {
        return;
    }

    // Create the image
    let size = Extent3d {
        width: MAP_SIZE as u32,
        height: MAP_SIZE as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
        | bevy::render::render_resource::TextureUsages::COPY_DST;
    let image_handle = images.add(image);

    commands.insert_resource(DebugMapImageHandle(image_handle.clone()));

    // Root container — covers entire screen, semi-transparent background
    commands
        .spawn((
            DebugMapRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // Info text at the top
            parent.spawn((
                DebugMapText,
                Text::new("Debug Map"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // The map image
            parent.spawn((
                DebugMapImage,
                ImageNode::new(image_handle),
                Node {
                    width: Val::Px(512.0),
                    height: Val::Px(512.0),
                    ..default()
                },
            ));
        });
}

/// Despawn the debug map UI when it becomes closed.
pub fn despawn_debug_map_ui(
    mut commands: Commands,
    state: Res<DebugMapState>,
    roots: Query<Entity, With<DebugMapRoot>>,
) {
    if state.open {
        return;
    }
    for entity in roots.iter() {
        commands.entity(entity).despawn();
    }
}

/// Regenerate the chunk cache when the map is dirty.
pub fn regenerate_debug_map(
    mut state: ResMut<DebugMapState>,
    player_q: Query<&Transform, With<Player>>,
    handle_res: Option<Res<DebugMapImageHandle>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !state.open || !state.dirty {
        return;
    }
    if handle_res.is_none() {
        return;
    }
    let handle = &handle_res.as_ref().unwrap().0;

    // Get player position for centering
    let player_pos = if let Ok(tf) = player_q.single() {
        tf.translation
    } else {
        Vec3::ZERO
    };

    let center_x = player_pos.x.floor() as i32;
    let center_z = player_pos.z.floor() as i32;
    let half = (MAP_SIZE / 2) as i32;

    // Origin in world block coords (top-left corner of map)
    let origin_x = center_x - half;
    let origin_z = center_z - half;

    // Chunk range in XZ
    let chunk_x_min = origin_x.div_euclid(CHUNK_SIZE as i32);
    let chunk_z_min = origin_z.div_euclid(CHUNK_SIZE as i32);
    // We need chunks that cover origin..(origin + MAP_SIZE)
    let chunk_x_max = (origin_x + MAP_SIZE as i32 - 1).div_euclid(CHUNK_SIZE as i32);
    let chunk_z_max = (origin_z + MAP_SIZE as i32 - 1).div_euclid(CHUNK_SIZE as i32);

    let cx_count = (chunk_x_max - chunk_x_min + 1) as usize;
    let cz_count = (chunk_z_max - chunk_z_min + 1) as usize;

    // Chunk range in Y: we need 0..16 (world Y 0..255)
    let chunk_y_min = 0i32;
    let chunk_y_max = 15i32;
    let cy_count = (chunk_y_max - chunk_y_min + 1) as usize;

    // Only regenerate chunks if center changed significantly
    let needs_regen = state.chunks.is_empty()
        || state.chunk_x_origin != chunk_x_min
        || state.chunk_z_origin != chunk_z_min
        || (state.center_x - center_x).abs() > 8
        || (state.center_z - center_z).abs() > 8;

    if needs_regen {
        // Generate all needed chunks
        let mut chunks: Vec<Vec<Vec<Chunk>>> = Vec::with_capacity(cy_count);
        for cy_idx in 0..cy_count {
            let cy = chunk_y_min + cy_idx as i32;
            let mut z_row: Vec<Vec<Chunk>> = Vec::with_capacity(cz_count);
            for cz_idx in 0..cz_count {
                let cz = chunk_z_min + cz_idx as i32;
                let mut x_row: Vec<Chunk> = Vec::with_capacity(cx_count);
                for cx_idx in 0..cx_count {
                    let cx = chunk_x_min + cx_idx as i32;
                    x_row.push(generate_chunk(IVec3::new(cx, cy, cz)));
                }
                z_row.push(x_row);
            }
            chunks.push(z_row);
        }

        // Build heightmap
        let mut heightmap = vec![vec![0i32; MAP_SIZE]; MAP_SIZE];
        for pz in 0..MAP_SIZE {
            let wz = origin_z + pz as i32;
            for px in 0..MAP_SIZE {
                let wx = origin_x + px as i32;
                // Scan from top down to find first non-air block
                let mut found_height = 0i32;
                for wy in (0..=255).rev() {
                    let cy_idx = (wy / CHUNK_SIZE as i32 - chunk_y_min) as usize;
                    if cy_idx >= cy_count {
                        continue;
                    }
                    let local_y = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
                    let cz_idx = (wz.div_euclid(CHUNK_SIZE as i32) - chunk_z_min) as usize;
                    let cx_idx = (wx.div_euclid(CHUNK_SIZE as i32) - chunk_x_min) as usize;
                    if cz_idx >= cz_count || cx_idx >= cx_count {
                        continue;
                    }
                    let local_x = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
                    let local_z = wz.rem_euclid(CHUNK_SIZE as i32) as usize;
                    let block = chunks[cy_idx][cz_idx][cx_idx].get(local_x, local_y, local_z);
                    if block != BlockType::Air && block != BlockType::Water {
                        found_height = wy;
                        break;
                    }
                }
                heightmap[pz][px] = found_height;
            }
        }

        state.chunks = chunks;
        state.heightmap = heightmap;
        state.chunk_x_origin = chunk_x_min;
        state.chunk_z_origin = chunk_z_min;
        state.center_x = center_x;
        state.center_z = center_z;
        state.chunk_y_min = chunk_y_min;
        state.chunk_y_max = chunk_y_max;
    }

    // Now render the map into the image
    let Some(image) = images.get_mut(handle) else {
        return;
    };

    let Some(ref mut pixel_data) = image.data else {
        return;
    };

    let y_level = state.y_level;
    let origin_x = state.center_x - (MAP_SIZE / 2) as i32;
    let origin_z = state.center_z - (MAP_SIZE / 2) as i32;
    let chunk_x_min = state.chunk_x_origin;
    let chunk_z_min = state.chunk_z_origin;
    let cy_count = state.chunks.len();
    let cz_count = if cy_count > 0 { state.chunks[0].len() } else { 0 };
    let cx_count = if cz_count > 0 && cy_count > 0 { state.chunks[0][0].len() } else { 0 };

    for pz in 0..MAP_SIZE {
        let wz = origin_z + pz as i32;
        for px in 0..MAP_SIZE {
            let wx = origin_x + px as i32;
            let pixel_idx = (pz * MAP_SIZE + px) * 4;

            let color = match state.mode {
                DebugMapMode::Block => {
                    let block = sample_block(
                        &state.chunks, wx, y_level, wz,
                        chunk_x_min, state.chunk_y_min, chunk_z_min,
                        cx_count, cy_count, cz_count,
                    );
                    block_color(block)
                }
                DebugMapMode::Heightmap => {
                    let h = state.heightmap[pz][px];
                    // Map height 0..128 to brightness 0..255
                    let brightness = ((h as f32 / 128.0) * 255.0).clamp(0.0, 255.0) as u8;
                    [brightness, brightness, brightness, 255]
                }
                DebugMapMode::Cave => {
                    let block = sample_block(
                        &state.chunks, wx, y_level, wz,
                        chunk_x_min, state.chunk_y_min, chunk_z_min,
                        cx_count, cy_count, cz_count,
                    );
                    let terrain_h = state.heightmap[pz][px];
                    if y_level >= terrain_h {
                        // Above terrain — dark blue to distinguish from underground
                        [20, 20, 40, 255]
                    } else if block == BlockType::Air || block == BlockType::Water {
                        // Underground air = cave
                        if block == BlockType::Water {
                            [30, 80, 200, 255]
                        } else {
                            [220, 220, 220, 255]
                        }
                    } else {
                        // Solid underground
                        [16, 16, 16, 255]
                    }
                }
                DebugMapMode::Biome => {
                    // Approximate biome from heightmap and surface block
                    // Check what the surface block is at this column
                    let h = state.heightmap[pz][px];
                    let surface_block = sample_block(
                        &state.chunks, wx, h, wz,
                        chunk_x_min, state.chunk_y_min, chunk_z_min,
                        cx_count, cy_count, cz_count,
                    );
                    match surface_block {
                        BlockType::Sand | BlockType::Sandstone => [237, 201, 100, 255], // desert yellow
                        BlockType::Grass | BlockType::TallGrass => [50, 180, 50, 255], // plains green
                        BlockType::Water => [30, 80, 200, 255],
                        BlockType::Snow => [230, 230, 255, 255],
                        _ => [100, 160, 80, 255], // generic green
                    }
                }
            };

            if pixel_idx + 3 < pixel_data.len() {
                pixel_data[pixel_idx] = color[0];
                pixel_data[pixel_idx + 1] = color[1];
                pixel_data[pixel_idx + 2] = color[2];
                pixel_data[pixel_idx + 3] = color[3];
            }
        }
    }

    state.dirty = false;
}

/// Sample a block from the cached chunk data at world coordinates.
fn sample_block(
    chunks: &[Vec<Vec<Chunk>>],
    wx: i32, wy: i32, wz: i32,
    chunk_x_min: i32, chunk_y_min: i32, chunk_z_min: i32,
    cx_count: usize, cy_count: usize, cz_count: usize,
) -> BlockType {
    if wy < 0 || wy > 255 {
        return BlockType::Air;
    }
    let cy_idx = (wy.div_euclid(CHUNK_SIZE as i32) - chunk_y_min) as usize;
    let cz_idx = (wz.div_euclid(CHUNK_SIZE as i32) - chunk_z_min) as usize;
    let cx_idx = (wx.div_euclid(CHUNK_SIZE as i32) - chunk_x_min) as usize;

    if cy_idx >= cy_count || cz_idx >= cz_count || cx_idx >= cx_count {
        return BlockType::Air;
    }

    let local_x = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_y = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
    let local_z = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

    chunks[cy_idx][cz_idx][cx_idx].get(local_x, local_y, local_z)
}

/// Update the info text overlay.
pub fn update_debug_map_text(
    state: Res<DebugMapState>,
    mut text_q: Query<&mut Text, With<DebugMapText>>,
) {
    if !state.open {
        return;
    }
    for mut text in text_q.iter_mut() {
        *text = Text::new(format!(
            "Debug Map  |  Y: {}  |  Mode: {}  |  Center: ({}, {})\n[Up/Down] Y level  [Tab] Mode  [F4] Close",
            state.y_level,
            state.mode.label(),
            state.center_x,
            state.center_z,
        ));
    }
}
