# MineClaude - Project Specification

A Minecraft clone built in Rust with the Bevy 0.18 game engine.

---

## Technology Stack

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | stable (2024 edition) |
| Game Engine | Bevy | 0.18 |
| Noise | noise crate | 0.9 |
| RNG | rand | 0.9 |
| Serialization | serde + serde_json + bincode + flate2 | latest |
| Atlas build | image crate (build-dep) | 0.25 |
| Linker | clang + lld | system |

### Cargo.toml Dependencies

```toml
[package]
name = "mineclaude"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { version = "0.18", features = ["free_camera"] }
noise = "0.9"
rand = "0.9"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
bincode = "1"
flate2 = "1"

[build-dependencies]
image = { version = "0.25", default-features = false, features = ["png"] }

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
opt-level = 3
lto = "thin"
```

### .cargo/config.toml

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-Clink-arg=-fuse-ld=lld"]
```

---

## Project Module Structure

```
src/
├── main.rs                     # App setup, plugin registration, game states
├── block/
│   ├── mod.rs                  # BlockType enum (28 types), BlockProperties, registry
│   └── atlas.rs                # Texture atlas UV mapping
├── world/
│   ├── mod.rs                  # WorldPlugin, world-level resources
│   ├── chunk.rs                # Chunk data structure (flat array)
│   ├── generation.rs           # Terrain gen: noise heightmap, caves (cheese/spaghetti/noodle), ores, trees
│   ├── meshing.rs              # Greedy meshing algorithm, face culling
│   ├── manager.rs              # Chunk loading/unloading, priority queue
│   └── coordinates.rs          # World/chunk/local coordinate conversions
├── player/
│   ├── mod.rs                  # PlayerPlugin
│   ├── controller.rs           # First-person camera, WASD movement, mouse look, combat
│   ├── interaction.rs          # Block breaking/placing, raycasting, chest/furnace/bed interaction
│   └── physics.rs              # AABB collision, gravity, jump, movement physics
├── entity/
│   ├── mod.rs                  # EntityPlugin
│   └── mob.rs                  # Mob types (sheep, cow, zombie, skeleton), AI, spawning, combat
├── inventory/
│   ├── mod.rs                  # InventoryPlugin
│   ├── inventory.rs            # Inventory data structure (36 slots), stack operations
│   ├── item.rs                 # Item enum (blocks + materials + tools), ToolKind, ToolTier
│   ├── crafting.rs             # 2x2 + 3x3 crafting grids, recipe matching
│   ├── furnace.rs              # Furnace smelting logic, fuel system
│   └── chest.rs                # Chest storage (27 slots per chest)
├── ui/
│   ├── mod.rs                  # UiPlugin
│   ├── hud.rs                  # Crosshair, debug overlay (F3)
│   ├── hotbar.rs               # Hotbar rendering, durability bars, item names
│   ├── inventory_screen.rs     # Inventory/crafting UI with 2x2 grid
│   ├── crafting_table_screen.rs # 3x3 crafting table UI
│   ├── furnace_screen.rs       # Furnace UI (input/fuel/output slots)
│   ├── chest_screen.rs         # Chest UI (27 slots + player inventory)
│   ├── main_menu.rs            # Main menu
│   └── pause_menu.rs           # Pause menu
├── lighting/
│   ├── mod.rs                  # LightingPlugin
│   ├── day_night.rs            # Day/night cycle, sun rotation, ambient adjustment
│   └── sky.rs                  # Procedural sky gradient
├── audio/
│   └── mod.rs                  # AudioPlugin — block break/place, footsteps
└── save/
    ├── mod.rs                  # SavePlugin
    └── persistence.rs          # Region-based save/load (bincode + gzip), player data
```

---

## Game States

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    Playing,    // Normal gameplay
    Paused,     // Pause menu open
    Inventory,  // Any UI screen open (inventory, crafting table, furnace, chest)
}
```

UI screen routing is handled by resources (`InventoryOpen`, `CraftingTableOpen`, `FurnaceOpen`, `ChestOpen`) rather than separate game states.

---

## Block Types (28 types)

| ID | Block | Solid | Transparent | Break Time (s) | Top Tex | Side Tex | Bottom Tex |
|----|-------|-------|-------------|-----------------|---------|----------|------------|
| 0 | Air | No | Yes | — | — | — | — |
| 1 | Stone | Yes | No | 7.5 | stone | stone | stone |
| 2 | Dirt | Yes | No | 0.75 | dirt | dirt | dirt |
| 3 | Grass | Yes | No | 0.9 | grass_top | grass_side | dirt |
| 4 | Cobblestone | Yes | No | 10.0 | cobblestone | cobblestone | cobblestone |
| 5 | Wood Planks | Yes | No | 3.0 | planks_oak | planks_oak | planks_oak |
| 6 | Sand | Yes | No | 0.75 | sand | sand | sand |
| 7 | Gravel | Yes | No | 0.9 | gravel | gravel | gravel |
| 8 | Oak Log | Yes | No | 3.0 | log_oak_top | log_oak | log_oak_top |
| 9 | Oak Leaves | Yes | Yes | 0.3 | leaves_oak | leaves_oak | leaves_oak |
| 10 | Glass | Yes | Yes | 0.45 | glass | glass | glass |
| 11 | Coal Ore | Yes | No | 15.0 | coal_ore | coal_ore | coal_ore |
| 12 | Iron Ore | Yes | No | 15.0 | iron_ore | iron_ore | iron_ore |
| 13 | Gold Ore | Yes | No | 15.0 | gold_ore | gold_ore | gold_ore |
| 14 | Diamond Ore | Yes | No | 15.0 | diamond_ore | diamond_ore | diamond_ore |
| 15 | Bedrock | Yes | No | ∞ | bedrock | bedrock | bedrock |
| 16 | Water | No | Yes | — | water | water | water |
| 17 | Crafting Table | Yes | No | 3.75 | crafting_top | crafting_side | planks_oak |
| 18 | Furnace | Yes | No | 17.5 | furnace_top | furnace_front | furnace_side |
| 19 | Torch | No | Yes | 0.0 | — | — | — |
| 20 | Snow | Yes | No | 0.3 | snow | snow | snow |
| 21 | Clay | Yes | No | 0.9 | clay | clay | clay |
| 22 | Sandstone | Yes | No | 4.0 | sandstone_top | sandstone | sandstone_bottom |
| 23 | Birch Log | Yes | No | 3.0 | log_birch_top | log_birch | log_birch_top |
| 24 | Birch Leaves | Yes | Yes | 0.3 | leaves_birch | leaves_birch | leaves_birch |
| 25 | Tall Grass | No | Yes | 0.0 | — | — | — |
| 26 | Chest | Yes | No | 3.75 | planks_oak | planks_oak | planks_oak |
| 27 | Bed | Yes | No | 0.3 | dirt | dirt | dirt |

---

## Items

Items are either block references or standalone materials/tools.

### Material Items

| Item | Stack Size |
|------|-----------|
| Stick | 64 |
| Coal | 64 |
| Iron Ingot | 64 |
| Gold Ingot | 64 |
| Diamond | 64 |

### Tool Items

All tools have stack size 1 and per-tier durability.

| Tool | Wooden | Stone | Iron | Diamond |
|------|--------|-------|------|---------|
| Pickaxe | 59 dur | 131 dur | 250 dur | 1561 dur |
| Axe | 59 dur | 131 dur | 250 dur | 1561 dur |
| Shovel | 59 dur | 131 dur | 250 dur | 1561 dur |
| Sword | 59 dur | 131 dur | 250 dur | 1561 dur |

**Tool speed multipliers** (base break time is divided by these, vanilla-accurate):
- Hand: 1 (no speed bonus)
- Wooden: 2
- Stone: 4
- Iron: 6
- Diamond: 8
- Gold: 12

**Tool requirements:**
- Iron ore: stone pickaxe or better
- Gold ore: iron pickaxe or better
- Diamond ore: iron pickaxe or better

**Weapon damage:**
- Hand: 1.0
- Wooden sword: 4.0
- Stone sword: 5.0
- Iron sword: 6.0
- Diamond sword: 7.0

---

## Crafting Recipes

### 2x2 Grid (Player Inventory)

| Output | Pattern |
|--------|---------|
| 4 Planks | 1 Log (any) |
| 4 Sticks | 2 Planks vertical |
| 1 Crafting Table | 2x2 Planks |
| 1 Torch | Stick below Coal |

### 3x3 Grid (Crafting Table)

| Output | Pattern |
|--------|---------|
| 1 Furnace | 8 Cobblestone ring (hollow center) |
| 1 Chest | 8 Planks ring (hollow center) |
| 4 Sandstone | 2x2 Sand |
| 1 Torch | Coal over Stick (center column) |
| 1 Bed | 3 Dirt top row + 3 Planks bottom row |
| Wooden tools | Standard Minecraft patterns (planks + sticks) |
| Stone tools | Standard Minecraft patterns (cobble + sticks) |
| Iron tools | Standard Minecraft patterns (iron ingot + sticks) |
| Diamond tools | Standard Minecraft patterns (diamond + sticks) |

### Smelting (Furnace)

| Input | Output | Smelt Time |
|-------|--------|-----------|
| Iron Ore | Iron Ingot | 10s |
| Gold Ore | Gold Ingot | 10s |
| Coal Ore | Coal | 10s |
| Cobblestone | Stone | 10s |
| Sand | Glass | 10s |

**Fuel values:** Coal = 80s, Log/Planks = 15s, Stick = 5s

---

## Mobs

### Passive Mobs
| Mob | HP | Spawns | Drops |
|-----|-----|--------|-------|
| Sheep | 8 | Daytime, on grass | — |
| Cow | 10 | Daytime, on grass | — |

### Hostile Mobs
| Mob | HP | Damage | Spawns | Behavior | Drops |
|-----|-----|--------|--------|----------|-------|
| Zombie | 20 | 3.0 | Night only | Chase within 16 blocks, lose at 24 | Sticks |
| Skeleton | 20 | 2.0 | Night only | Chase within 16 blocks, lose at 24 | Sticks |

- Hostile mobs burn in sunlight when sky-exposed
- Despawn at 120 blocks distance
- Attack cooldown: melee AI

---

## Survival Mechanics

### Health
- 20 HP (10 hearts)
- Fall damage: 1 HP per block after 3-block safe distance
- Drowning: air supply depletes underwater, then HP drain
- Death respawn at bed spawn point or world default (0, 80, 0)

### Combat
- Player attack via ray-AABB hit detection
- Attack cooldown: 0.5s
- Knockback: 8 horizontal + 4 vertical
- Tool durability consumed on hit
- Mob drops require player within 16 blocks

### Storage
- Chest: 27-slot persistent storage (data not yet saved to disk)
- Bed: sets spawn point, skips night

---

## Terrain Generation

### Heightmap
- Multi-octave 2D Perlin noise (Fbm, 3 octaves)
- Base height range: 40-100 blocks
- Biomes: Plains, Desert (temperature/humidity noise)

### Surface Layers
- Grass on top, 3-4 dirt below, stone underneath
- Desert: sand surface, sandstone subsurface
- Oak trees (~70%) and birch trees (~30%) in plains
- Tall grass on grass blocks in plains
- Gravel patches underground
- Clay patches near water

### Caves (Modern Minecraft-style)
- **Cheese caves**: Large 3D noise caverns
- **Spaghetti caves**: Narrow winding tunnels (two noise fields intersected)
- **Noodle caves**: Thinner variant of spaghetti

### Ores (Triangular Y-distribution)
- Coal: Y 0-128, peak 96
- Iron: Y 0-64, peak 32
- Gold: Y 0-32, peak 16
- Diamond: Y 0-16, peak 8

### Other
- Water fill below sea level (Y=63)
- Bedrock at Y=0

---

## Persistence

### Save Format
- Region-based chunk storage (bincode + gzip)
- Save directory: `saves/`
- Auto-save + manual save (Ctrl+S)
- Modified chunk tracking (only save dirty chunks)
- Chunks saved on despawn

### Player Data (JSON)
- Position, rotation
- Health, air supply
- Inventory contents
- Spawn point (bed location)

### Not Yet Persisted
- Chest contents
- Furnace state (input/fuel/output, progress)

---

## Core Architecture Decisions

### Chunk System
- **Chunk size**: 16x16x16 blocks (one section)
- **World height**: 256 blocks (16 sections stacked, Y=0 to Y=255)
- **Storage**: Flat `[u8; 4096]` array per chunk, YZX ordering
- **BlockId**: `u8` (supports 256 block types)
- **Coordinate conversion**: Always use `div_euclid`/`rem_euclid` for negative coords

### World Layout
- Chunks addressed by `IVec3` (chunk_x, chunk_y, chunk_z)
- World column = 16 chunks stacked vertically (Y=0..15)
- Sea level at Y=63 (block Y, which is chunk Y=3, local Y=15)
- Render distance: 16 chunks horizontal
- Despawn distance: 18 chunks

### Meshing Strategy
- Custom greedy meshing implementation (currently disabled — using 1x1 quads)
- Face culling: only emit faces between solid↔non-solid boundaries
- Cross-chunk culling: padding approach (read 1-block border from neighbors)
- Async meshing via `AsyncComputeTaskPool`
- One mesh entity per chunk, one shared `StandardMaterial` with texture atlas
- Nearest-neighbor texture filtering for pixel art style

### Player Physics (Minecraft values)
- Walk: 4.317 m/s
- Sprint: 5.612 m/s (1.3x multiplier)
- Sneak: 1.295 m/s (0.3x multiplier)
- Gravity: 20 m/s² (Euler integration with frame delta)
- Jump velocity: 7.4 m/s
- Player hitbox: 0.6 x 1.8 x 0.6 (W x H x D)
- Eye height: 1.62

### Bevy 0.18 API Notes
- Events renamed to Messages: `#[derive(Message)]`, `MessageWriter`, `MessageReader`
- Import from `bevy::ecs::message::{Message, MessageReader, MessageWriter}`
- `GlobalAmbientLight` resource for scene-wide ambient (not per-camera `AmbientLight`)
- `AccumulatedMouseMotion` for mouse delta
- `block_on(poll_once(&mut task))` from `bevy::tasks`

---

## Texture Atlas Layout

- Atlas size: 256x256 pixels (16x16 grid of 16x16 tiles)
- 256 tile slots total
- UV computation:
  ```
  tile_col = tile_index % 16
  tile_row = tile_index / 16
  u_min = tile_col / 16.0
  v_min = tile_row / 16.0
  u_max = u_min + 1.0/16.0
  v_max = v_min + 1.0/16.0
  ```
- Source: ProgrammerArt (CC BY 4.0) from https://github.com/deathcap/ProgrammerArt

---

## Assets

### Textures
- **Primary**: ProgrammerArt — CC BY 4.0 — 16x16 Minecraft-compatible textures
- **Atlas**: Build-time script (`build.rs`) stitches individual PNGs into `atlas.png`

### Sounds
- Block SFX: CC0 from OpenGameArt (breaking/hit SFX)
- Footsteps: CC0 from OpenGameArt (multi-surface footstep packs)

### Font
- monogram pixel font — CC0 — from https://datagoblin.itch.io/monogram

### Sky
- Procedural sky gradient (day/night color interpolation)

---

## Performance Targets

| Metric | Target |
|--------|--------|
| FPS | 60+ at render distance 16 |
| Chunk mesh time | <5ms per chunk (async) |
| Chunk load rate | 4-8 chunks/frame |
| Compile time (incremental) | <5s with lld + dynamic linking |

---

## Key Constants

```rust
// World
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE; // 4096
pub const WORLD_HEIGHT_CHUNKS: i32 = 16;  // 256 blocks
pub const SEA_LEVEL: i32 = 63;
pub const RENDER_DISTANCE: i32 = 16;
pub const DESPAWN_DISTANCE: i32 = 18;

// Player
pub const PLAYER_WIDTH: f32 = 0.6;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_EYE_HEIGHT: f32 = 1.62;
pub const REACH_DISTANCE: f32 = 5.0;

// Day/Night
pub const DAY_LENGTH_SECONDS: f32 = 1200.0; // 20 minutes

// Inventory
pub const HOTBAR_SLOTS: usize = 9;
pub const MAIN_INVENTORY_SLOTS: usize = 27;
pub const DEFAULT_STACK_SIZE: u32 = 64;

// Combat
pub const ATTACK_COOLDOWN: f32 = 0.5;
pub const KNOCKBACK_HORIZONTAL: f32 = 8.0;
pub const KNOCKBACK_VERTICAL: f32 = 4.0;

// Mobs
pub const HOSTILE_CHASE_RANGE: f32 = 16.0;
pub const HOSTILE_LOSE_RANGE: f32 = 24.0;
pub const MOB_DESPAWN_DISTANCE: f32 = 120.0;

// Lighting
pub const MAX_LIGHT_LEVEL: u8 = 15;
```

---

## Known Issues

- Greedy meshing disabled — 1x1 quads at RD 16 is a performance concern
- Chest/furnace data not persisted to disk — items lost on save/load
- Chest/Bed use placeholder textures (planks/dirt)
- Bed recipe uses Dirt as wool placeholder
- Ground collision is single-point — player can fall through block corners
- No horizontal drag — instant stop, no momentum
- Skeleton is melee-only (should have ranged bow attack)
- Mob pathfinding walks through walls
- ore_noises[0] seed collides with cave_cheese (both SEED+10)
- No dropped item entities — items teleport directly to inventory (no visible pickup)
- Crafting table screen missing player inventory slots (can't use inventory items)
- Furnace screen may have same issue — no player inventory shown
- Camera moves when in-game menus are open (crafting table, possibly furnace)
- UI code duplication — slot interaction, item colors copied across 5+ screens; needs shared UI abstraction
- break_block system has 12 params — close to Bevy's limit

---

## Future Priorities

### High
- Hunger system + food items
- Door block (seal shelters from mobs)
- Chest/furnace persistence to disk
- Fix greedy meshing

### Medium
- Armor (leather, iron, diamond)
- Mob sounds
- Sapling drops + tree growing (renewable wood)
- Farming (seeds, wheat, farmland)

### Low
- Bow + arrows, bucket, ladders, fences, stairs/slabs
- Cave ambient sounds, background music
- Villages, dungeons, mineshafts
- Creeper, spider, enderman
