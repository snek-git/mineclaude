# Bevy 0.18 Engine Research for Minecraft Clone

## 1. Bevy 0.18 Features Overview

Released January 13, 2026. 174 contributors, 659 pull requests.

### Key New Features
- **Atmosphere Occlusion & PBR Shading** - Procedural atmosphere affects how light reaches objects; sunlight picks up realistic colors through the atmosphere
- **Generalized Atmospheric Scattering Media** - `ScatteringMedium` assets for customizable atmospheric effects
- **PBR Shading Fixes** - Fixed overly glossy materials, switched from roughness-dependent Fresnel to direct Fresnel terms
- **Solari Improvements** - Experimental raytraced renderer gained specular materials, faster lighting, physically-based soft shadows
- **First-Party Camera Controllers** - Built-in `FreeCamera` and `PanCamera` (see section 2)
- **Fullscreen Materials** - `FullscreenMaterial` trait for post-processing shaders
- **Cargo Feature Collections** - High-level feature sets (`2d`, `3d`, `ui`) for lightweight builds
- **Standard UI Widgets** - `Popover`, `MenuPopup` with keyboard navigation
- **Font Variations** - Variable weight fonts, strikethroughs, underlines, OpenType features
- **Safe Component Access** - `get_components_mut` for multiple arbitrary components with runtime aliasing checks
- **Schedule Management** - `remove_systems_in_set` to completely remove systems
- **glTF Extensions** - `GltfExtensionHandler` for custom extension processing
- **Easy Screenshots/Recording** - `EasyScreenshotPlugin` and `EasyScreenRecordPlugin`

### Stable APIs
The core ECS, rendering, asset, input, and audio APIs are stable and well-documented. Mesh generation, material system, and transform hierarchy are mature.

---

## 2. First-Party Camera Controllers

Bevy 0.18 ships with built-in camera controllers. No more need for third-party crates like `bevy_flycam`.

### FreeCamera (Fly Camera)
Requires the `free_camera` cargo feature:

```toml
[dependencies]
bevy = { version = "0.18", features = ["free_camera"] }
```

```rust
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};

#[derive(Component)]
#[require(Camera3d, FreeCamera)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Transform::from_xyz(0.0, 60.0, 0.0),
    ));
}
```

**Default keybindings:** W/A/S/D for horizontal movement, E/Q for up/down, mouse for look.

> **Note for Minecraft clone:** We will likely want a custom camera controller rather than FreeCamera, since Minecraft has gravity-based player movement, not noclip. FreeCamera is useful for development/debug mode only.

### PanCamera
2D pan-zoom camera with scroll wheel zoom. Not relevant for our 3D voxel game.

---

## 3. Custom Mesh Generation

This is the core API we need for building chunk meshes from voxel data.

### Creating a Mesh from Vertices

```rust
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

fn create_voxel_face() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![
                [0.0, 1.0],
                [1.0, 1.0],
                [1.0, 0.0],
                [0.0, 0.0],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
        )
        .with_inserted_indices(Indices::U32(vec![
            0, 1, 2,
            0, 2, 3,
        ]))
}
```

### Key Mesh API

- `Mesh::new(topology, usage)` - Create empty mesh with topology (TriangleList for voxels)
- `.with_inserted_attribute(id, data)` - Add vertex attribute (position, UV, normal)
- `.with_inserted_indices(Indices::U32(vec))` - Set triangle indices
- `Mesh::ATTRIBUTE_POSITION` - `[f32; 3]` vertex positions
- `Mesh::ATTRIBUTE_NORMAL` - `[f32; 3]` normals per vertex
- `Mesh::ATTRIBUTE_UV_0` - `[f32; 2]` texture coordinates
- `Mesh::ATTRIBUTE_COLOR` - `[f32; 4]` per-vertex color (useful for AO)
- `.compute_smooth_normals()` / `.compute_flat_normals()` - Auto-compute normals
- `RenderAssetUsages::default()` - Makes mesh available to both main world and render world

### Spawning a Mesh Entity

```rust
fn spawn_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = create_chunk_mesh(/* chunk data */);
    let mesh_handle = meshes.add(mesh);

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            ..default()
        })),
        Transform::from_translation(chunk_world_position),
    ));
}
```

### Updating Meshes (for chunk remeshing)

```rust
fn update_chunk_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh3d, With<ChunkMarker>>,
) {
    for mesh_handle in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            // Replace mesh data in-place - no need to despawn/respawn
            *mesh = generate_new_mesh(/* updated chunk data */);
        }
    }
}
```

---

## 4. Texture Atlas Handling

### TextureAtlasLayout for Sprite Sheets

For 2D sprite atlases, Bevy provides `TextureAtlasLayout`:

```rust
fn setup_atlas(
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("textures/blocks.png");
    // Grid of 16x16 pixel tiles, 16 columns, 16 rows
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(16), // tile size
        16,               // columns
        16,               // rows
        None,             // padding
        None,             // offset
    );
    let layout_handle = texture_atlas_layouts.add(layout);
}
```

### UV Mapping for 3D Voxel Meshes (Our Approach)

For 3D voxel chunk meshes, we do NOT use Bevy's TextureAtlas system directly. Instead, we load a single texture atlas image and compute UVs manually:

```rust
const ATLAS_SIZE: f32 = 256.0;  // Total atlas size in pixels
const TILE_SIZE: f32 = 16.0;    // Each block texture is 16x16
const TILES_PER_ROW: f32 = ATLAS_SIZE / TILE_SIZE; // 16 tiles per row

fn tile_uv(tile_index: u32) -> [[f32; 2]; 4] {
    let col = (tile_index % TILES_PER_ROW as u32) as f32;
    let row = (tile_index / TILES_PER_ROW as u32) as f32;

    let u_min = col * TILE_SIZE / ATLAS_SIZE;
    let v_min = row * TILE_SIZE / ATLAS_SIZE;
    let u_max = u_min + TILE_SIZE / ATLAS_SIZE;
    let v_max = v_min + TILE_SIZE / ATLAS_SIZE;

    [
        [u_min, v_max], // bottom-left
        [u_max, v_max], // bottom-right
        [u_max, v_min], // top-right
        [u_min, v_min], // top-left
    ]
}
```

### StandardMaterial with Texture Atlas

```rust
fn create_block_material(
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/block_atlas.png")),
        perceptual_roughness: 1.0,  // Blocks are rough, not shiny
        reflectance: 0.1,           // Low reflectance for blocks
        // Use nearest-neighbor sampling for pixel art
        ..default()
    })
}
```

### Nearest-Neighbor Filtering for Pixel Art

Critical for Minecraft-style visuals. Configure image sampler:

```rust
app.add_plugins(DefaultPlugins.set(ImagePlugin {
    default_sampler: ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    },
}));
```

---

## 5. Chunk-Based Rendering

### Spawning/Despawning Chunk Entities

```rust
#[derive(Component)]
struct Chunk {
    position: IVec3,
}

fn spawn_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    chunk_pos: IVec3,
    mesh: Mesh,
) -> Entity {
    commands.spawn((
        Chunk { position: chunk_pos },
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(
            Vec3::new(
                (chunk_pos.x * CHUNK_SIZE) as f32,
                (chunk_pos.y * CHUNK_SIZE) as f32,
                (chunk_pos.z * CHUNK_SIZE) as f32,
            )
        ),
    )).id()
}

fn despawn_chunk(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}
```

### Batch Spawning for Performance

When loading many chunks at once:

```rust
// Use spawn_batch for large numbers of entities with the same component types
commands.spawn_batch(chunks_to_spawn.into_iter().map(|(pos, mesh_handle)| {
    (
        Chunk { position: pos },
        Mesh3d(mesh_handle),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(chunk_world_pos(pos)),
    )
}));
```

### Async Chunk Meshing

Use `AsyncComputeTaskPool` to generate meshes off the main thread:

```rust
use bevy::tasks::AsyncComputeTaskPool;

fn queue_chunk_meshing(
    pool: Res<AsyncComputeTaskPool>,
) {
    let task = pool.spawn(async move {
        // Heavy meshing work happens here, off main thread
        generate_chunk_mesh(chunk_data)
    });
    // Store task handle, poll in a system each frame
}
```

### LOD Strategies

For a Minecraft clone, simple LOD approach:
1. **Near chunks (0-4 chunks):** Full detail, all block faces
2. **Medium chunks (4-8 chunks):** Could skip small features (flowers, grass overlays)
3. **Far chunks (8-16 chunks):** Could use simplified meshes or skip underground chunks
4. **Beyond render distance:** Don't render at all

The `bevy_voxel_world` crate has built-in LOD support with configurable `chunk_lod()` callbacks.

---

## 6. Performance - Fast Compile Configuration

### .cargo/config.toml

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-Clink-arg=-fuse-ld=/usr/bin/mold", "-Zshare-generics=y", "-Zthreads=0"]

# Optional: Use cranelift for even faster dev builds (nightly only)
# [unstable]
# codegen-backend = true
#
# [profile.dev]
# codegen-backend = "cranelift"
#
# [profile.dev.package."*"]
# codegen-backend = "llvm"
```

### Cargo.toml Dev Profile

```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

This gives fast compiles for your code (opt-level 1) while keeping dependencies optimized (opt-level 3), which is critical for Bevy's rendering performance in debug builds.

### Dynamic Linking (Dev Only)

```bash
cargo run --features bevy/dynamic_linking
```

Or in Cargo.toml for development:

```toml
[features]
dev = ["bevy/dynamic_linking"]
```

Run with: `cargo run --features dev`

> **Do NOT ship with dynamic linking** - it requires bundling `libbevy_dylib` alongside the game.

### Mold Linker

Install on Arch/CachyOS: `pacman -S mold`

Mold is significantly faster than the default `ld` linker. Combined with dynamic linking and opt-level 1 for dev code, incremental rebuilds should be very fast.

---

## 7. Existing Voxel Crates Evaluation

### bevy_voxel_world (v0.15.0 for Bevy 0.18)

**Pros:**
- Supports Bevy 0.18
- Handles multithreaded meshing, chunk spawning/despawning, texture mapping
- Dual-layer voxel system (procedural + persistent)
- Built-in raycasting for block selection
- LOD support with configurable callbacks
- Custom meshing delegate support
- Multiple independent worlds via config types
- `set_voxel`/`get_voxel` API for runtime modification

**Cons:**
- Originated as internal game code, has some hard-coded assumptions
- Default meshing uses block-mesh-rs "simple" algorithm (no greedy meshing by default)
- Memory-heavy for large spawn distances (procedural layer caching)
- 286 stars - moderate community

**API:**
```rust
#[derive(Resource, Clone, Default)]
struct MyWorld;

impl VoxelWorldConfig for MyWorld {
    type MaterialIndex = u8;
    fn spawning_distance(&self) -> u32 { 25 }
    fn texture_index_mapper(&self) -> Arc<dyn Fn(u8) -> [u32; 3] + Send + Sync> {
        Arc::new(|material| match material {
            0 => [0, 1, 2], // top, sides, bottom texture indices
            _ => [3, 3, 3],
        })
    }
}

fn my_system(mut voxel_world: VoxelWorld<MyWorld>) {
    voxel_world.set_voxel(IVec3::new(0, 0, 0), WorldVoxel::Solid(0));
}
```

**Verdict:** Strong candidate if we want to move fast. Good API, supports Bevy 0.18, handles the boring plumbing (chunk lifecycle, threading, LOD). Custom meshing delegate means we can plug in our own mesher if needed.

### bevy_meshem (v0.5.x for Bevy 0.15)

**Pros:**
- Focused specifically on meshing algorithms
- VoxelRegistry trait for defining block types
- Face culling algorithm (Minecraft-style)
- O(1) mesh updates via MeshMD data structure
- Smooth lighting / ambient occlusion
- Adjacent chunk handling for seamless borders

**Cons:**
- Only supports up to Bevy 0.15 - NOT compatible with Bevy 0.18
- No greedy meshing (authors consider texture compromises unacceptable)
- Pre-release quality
- Meshing-only - no chunk management, no world loading

**Verdict:** Not usable - does not support Bevy 0.18. Would need significant porting effort.

### vx_bevy

**Pros:**
- Full Minecraft-style voxel prototype
- Greedy meshing via block-mesh crate
- Async chunk meshing with AsyncComputeTaskPool
- ~100 FPS on mid-range hardware at 16-chunk render distance

**Cons:**
- Only supports up to Bevy 0.14 - NOT compatible with Bevy 0.18
- "Won't receive further updates to newer bevy versions"
- Prototype quality, bugs (going underground breaks things)
- 92.3% Rust, 7.7% WGSL

**Verdict:** Not usable directly, but excellent reference for architecture patterns. The greedy meshing approach via `block-mesh` crate and async task pool pattern is worth studying.

### Recommendation

**Option A: Use bevy_voxel_world** as a foundation, customize with our own meshing delegate and game logic on top. Fastest path to a working demo. Risk: may hit limitations of the crate's architecture.

**Option B: Build from scratch** using patterns from vx_bevy and block-mesh-rs. More work upfront but full control. Use `block-mesh` crate for greedy meshing, implement our own chunk management with Bevy ECS.

**Suggested: Option A initially, with Option B as fallback.** Start with `bevy_voxel_world` for rapid prototyping. If we hit walls, we have enough understanding to build our own system.

---

## 8. Lighting System

### Light Types

Bevy provides PBR (Physically Based Rendering) lighting:

```rust
// Directional Light (Sun)
commands.spawn((
    DirectionalLight {
        color: Color::srgb(1.0, 0.95, 0.85), // Warm sunlight
        illuminance: 10000.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_rotation(Quat::from_euler(
        EulerRot::XYZ,
        -std::f32::consts::FRAC_PI_4, // 45 degree angle
        std::f32::consts::FRAC_PI_4,
        0.0,
    )),
));

// Ambient Light (fill light so shadows aren't pure black)
commands.insert_resource(AmbientLight {
    color: Color::srgb(0.6, 0.7, 1.0), // Slight blue tint like sky
    brightness: 200.0,
});

// Point Light (for torches, lava, etc.)
commands.spawn((
    PointLight {
        color: Color::srgb(1.0, 0.7, 0.3), // Warm torch light
        intensity: 1000.0,
        range: 15.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(5.0, 5.0, 5.0),
));

// Spot Light (less common for Minecraft, but available)
commands.spawn((
    SpotLight {
        color: Color::WHITE,
        intensity: 5000.0,
        range: 20.0,
        outer_angle: 0.5,
        inner_angle: 0.3,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(0.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
));
```

### Shadow Configuration

```rust
// Control cascade shadow maps for directional light
commands.spawn((
    DirectionalLight { shadows_enabled: true, ..default() },
    CascadeShadowConfig {
        num_cascades: 4,
        maximum_distance: 100.0,
        ..default()
    },
));
```

### Bevy 0.18 Atmosphere

New in 0.18 - procedural atmosphere affects lighting:
- Sunlight picks up realistic colors as it travels through the atmosphere
- Orange/red tones near the horizon (sunset/sunrise)
- `ScatteringMedium` assets for fog and atmospheric effects

### Day/Night Cycle Approach

Rotate the directional light transform over time and adjust color/intensity:

```rust
fn day_night_cycle(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut DirectionalLight)>,
) {
    for (mut transform, mut light) in &mut query {
        let day_progress = (time.elapsed_secs() / DAY_LENGTH) % 1.0;
        let angle = day_progress * std::f32::consts::TAU;
        transform.rotation = Quat::from_rotation_x(angle);

        // Adjust intensity based on sun height
        let sun_height = angle.sin();
        light.illuminance = (sun_height * 10000.0).max(0.0);
    }
}
```

---

## 9. Audio System

Bevy has built-in audio powered by the `rodio` library.

### Core Components
- `AudioPlayer<AudioSource>` - Component that triggers playback when spawned
- `AudioSink` - Auto-inserted by Bevy, used to control playback (pause, volume, speed)
- `SpatialAudioSink` - For positional audio (e.g., nearby sounds)
- `PlaybackSettings` - Configure looping, volume, spatial, speed
- `SpatialListener` - Camera/player component to receive spatial audio

### Playing Sound Effects

```rust
fn play_block_break(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("sounds/block_break.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Despawn, // Auto-despawn entity when done
            volume: Volume::new(0.5),
            ..default()
        },
    ));
}
```

### Background Music (Looping)

```rust
fn play_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("music/ambient.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.3),
            ..default()
        },
    ));
}
```

### Spatial Audio

```rust
// On the player/camera
commands.spawn((
    Camera3d::default(),
    SpatialListener::default(),
    Transform::from_xyz(0.0, 0.0, 0.0),
));

// On a sound source (e.g., water, lava, mob)
commands.spawn((
    AudioPlayer::<AudioSource>(asset_server.load("sounds/water.ogg")),
    PlaybackSettings {
        mode: PlaybackMode::Loop,
        spatial: true,
        ..default()
    },
    Transform::from_xyz(10.0, 5.0, 10.0),
));
```

### Controlling Playback

```rust
fn toggle_music(
    query: Query<&AudioSink, With<MusicMarker>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyM) {
        if let Ok(sink) = query.single() {
            sink.toggle();
        }
    }
}
```

> **Note:** Bevy's spatial audio is simple stereo panning (left-right). No HRTF or advanced 3D audio. Adequate for a Minecraft clone.

---

## 10. UI System

Bevy's UI is fully ECS-based, powered by the `taffy` layout library (Flexbox + CSS Grid).

### Core UI Components
- `Node` - Layout properties (width, height, flex, padding, margin, etc.)
- `Text` / `TextFont` / `TextColor` - Text rendering
- `BackgroundColor` - Node background
- `BorderColor` - Node border
- `ImageNode` - Image display
- `Button` / `Interaction` - Interactive elements
- `Val::Px(f32)` / `Val::Percent(f32)` / `Val::Vw(f32)` - Sizing units

### HUD Crosshair

```rust
fn spawn_crosshair(mut commands: Commands) {
    // Simple crosshair using a small centered node
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(2.0),
            height: Val::Px(20.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-1.0),
                top: Val::Px(-10.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(20.0),
            height: Val::Px(2.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-10.0),
                top: Val::Px(-1.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
}
```

### Hotbar

```rust
fn spawn_hotbar(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Percent(50.0),
            margin: UiRect { left: Val::Px(-180.0), ..default() }, // Center 9 slots
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(2.0),
            ..default()
        },
    )).with_children(|parent| {
        for i in 0..9 {
            parent.spawn((
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ));
        }
    });
}
```

### Health Bar

```rust
#[derive(Component)]
struct HealthBar;

fn spawn_health_bar(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            width: Val::Px(200.0),
            height: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
    )).with_children(|parent| {
        parent.spawn((
            HealthBar,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
        ));
    });
}

fn update_health_bar(
    player: Query<&Health, With<Player>>,
    mut bar: Query<&mut Node, With<HealthBar>>,
) {
    if let (Ok(health), Ok(mut node)) = (player.single(), bar.single_mut()) {
        node.width = Val::Percent(health.current / health.max * 100.0);
    }
}
```

### Debug Text Overlay

```rust
fn spawn_debug_text(mut commands: Commands) {
    commands.spawn((
        DebugText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        Text::new("FPS: 0"),
        TextFont::default().with_font_size(16.0),
        TextColor(Color::WHITE),
    ));
}
```

### New in 0.18
- `Popover` - Automatic popup positioning
- `MenuPopup` - Dropdown menus with keyboard navigation
- `AutoDirectionalNavigation` - Spatial UI navigation (gamepad/keyboard)
- Pickable text sections for tooltips/hyperlinks
- `IgnoreScroll` for sticky headers
- Color/layout interpolation with `TryStableInterpolate`

---

## 11. State Management

Bevy has a built-in state machine system.

### Defining States

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    Loading,
    Playing,
    Paused,
    Inventory,
}

// Register in app
app.insert_state(GameState::MainMenu);
```

### State-Dependent Systems

```rust
app.add_systems(Update, (
    handle_menu_input.run_if(in_state(GameState::MainMenu)),
    player_movement.run_if(in_state(GameState::Playing)),
    inventory_ui.run_if(in_state(GameState::Inventory)),
));

// Systems that run on state entry/exit
app.add_systems(OnEnter(GameState::Playing), (
    spawn_world,
    spawn_player,
    spawn_hud,
));

app.add_systems(OnExit(GameState::Playing), (
    despawn_world,
    despawn_hud,
));

app.add_systems(OnEnter(GameState::Paused), spawn_pause_menu);
app.add_systems(OnExit(GameState::Paused), despawn_pause_menu);
```

### Transitioning States

```rust
fn toggle_pause(
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}
```

### State Transition Order

Each frame, `StateTransition` schedule runs:
1. `StateTransitionEvent` sent
2. `OnExit(old_state)` schedule runs
3. `OnTransition { from, to }` schedule runs
4. `OnEnter(new_state)` schedule runs

### Multiple Orthogonal States

Can have independent state machines:

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum DebugState {
    #[default]
    Off,
    ShowFps,
    ShowChunkBorders,
    Full,
}

app.init_state::<DebugState>();

// Combine state conditions
app.add_systems(Update,
    show_fps.run_if(in_state(GameState::Playing)).run_if(in_state(DebugState::ShowFps)),
);
```

### SubStates

For states dependent on a parent state:

```rust
#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState = GameState::Playing)]
enum PlayingSubState {
    Exploring,
    Fighting,
    Trading,
}
```

---

## 12. Summary & Architecture Recommendations

### Recommended Stack for Minecraft Clone

| Feature | Approach |
|---------|----------|
| Rendering | Bevy 0.18 built-in PBR with StandardMaterial |
| Mesh Generation | Custom via Mesh API + block-mesh-rs for greedy meshing |
| Chunk Management | bevy_voxel_world (v0.15) OR custom ECS-based |
| Texture Atlas | Single image + manual UV computation |
| Camera | Custom player controller (not FreeCamera, need gravity) |
| Lighting | DirectionalLight (sun) + AmbientLight + PointLight (torches) |
| Audio | Built-in AudioPlayer with spatial audio |
| UI/HUD | Built-in bevy_ui (crosshair, hotbar, health) |
| State Machine | Built-in States (MainMenu, Loading, Playing, Paused) |
| Performance | mold linker + dynamic_linking + opt-level config |
| Image Filtering | Nearest-neighbor for pixel art style |

### Key Dependencies

```toml
[dependencies]
bevy = { version = "0.18", features = ["free_camera"] }
bevy_voxel_world = "0.15"  # Optional: if using as foundation
noise = "0.9"               # Terrain generation
rand = "0.8"                # Randomness

[dev-dependencies]
# None critical

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```
