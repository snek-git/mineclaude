# MineClaude

Minecraft clone in Rust + Bevy 0.18. See `SPEC.md` for full project specification.

## Key References
- `.claude/WORKFLOW.md` — **Development workflow**: continuous loop, team roles, watchdog, testing, quality gates
- `SPEC.md` — Master blueprint: architecture, phases, constants, block types
- `research/bevy_engine.md` — Bevy 0.18 API details, mesh generation, UI, audio, states
- `research/minecraft_mechanics.md` — Minecraft physics values, terrain gen, ore distribution
- `research/architecture.md` — Voxel engine patterns: greedy meshing, chunk management, collision
- `research/cave_generation.md` — Modern Minecraft cave generation (cheese/spaghetti/noodle)
- `research/assets.md` — Free asset sources with URLs and licenses

## Architecture

- Bevy ECS with one plugin per feature area (WorldPlugin, PlayerPlugin, UiPlugin, etc.)
- 16x16x16 chunks stored as flat `[u8; 4096]` arrays, YZX ordering
- Custom greedy meshing with async generation via `AsyncComputeTaskPool`
- Minecraft-accurate player physics (gravity=0.08, drag=0.98/0.91, jump=0.42)
- ProgrammerArt textures (CC BY 4.0) in a 256x256 atlas with nearest-neighbor filtering
- Coordinate conversions MUST use `div_euclid`/`rem_euclid` for negative coords

## Module Ownership (File Boundaries)

Each module is self-contained. Teammates MUST only edit files within their assigned module:

| Module | Files | Description |
|--------|-------|-------------|
| block | `src/block/mod.rs`, `src/block/atlas.rs` | Block types, properties, UV mapping |
| world/chunk | `src/world/chunk.rs`, `src/world/coordinates.rs` | Chunk data structure, coord math |
| world/gen | `src/world/generation.rs` | Terrain generation, caves, ores, trees |
| world/mesh | `src/world/meshing.rs` | Greedy meshing, face culling |
| world/mgr | `src/world/manager.rs`, `src/world/mod.rs` | Chunk load/unload, WorldPlugin |
| player | `src/player/*.rs` | Controller, physics, interaction |
| ui | `src/ui/*.rs` | HUD, menus, inventory screen |
| inventory | `src/inventory/*.rs` | Inventory data, crafting |
| lighting | `src/lighting/*.rs` | Day/night, light propagation |
| audio | `src/audio/*.rs` | Sound effects, music |
| save | `src/save/*.rs` | Region files, persistence |

## Coding Conventions

- Use `pub const` for game constants, defined in each module (not a central file)
- All Bevy systems must be `pub fn` with descriptive names: `update_chunk_loading`, `handle_block_break`
- Components: `#[derive(Component)]` with doc comments
- Resources: `#[derive(Resource)]` — prefer resources over globals
- No `unwrap()` in production code — use `if let`, `?`, or `.ok()`
- Keep systems focused: one responsibility per system function
- Plugin pattern: each module exports a `XxxPlugin` struct implementing `Plugin`

## Build & Run

```bash
# Dev (fast compile with dynamic linking)
cargo run --features bevy/dynamic_linking

# Release
cargo run --release
```

## Testing

- `cargo build` must succeed at all times — broken builds block everyone
- Test with `cargo run --features bevy/dynamic_linking` to verify visually
- When reporting build errors, include ONLY the first error message, not the full cascade
- Log format: `ERROR: [module] [description]` — one line, grep-friendly

## Team Workflow

**See `.claude/WORKFLOW.md` for the full continuous development workflow.**

Summary: Watchdog finds gaps → Lead prioritizes → Implementers fix → Verify → repeat.

1. Lead is a delegator — coordinates only, never writes code
2. Watchdog agent continuously scans for bugs, gaps, and Minecraft inaccuracies
3. Each teammate owns one module — no file conflicts
4. Teammate runs `cargo build` to verify before marking task complete
5. Tasks should be self-contained: one module, one deliverable, clear done criteria
6. If a task depends on another module's types/API, the dependency task must complete first
