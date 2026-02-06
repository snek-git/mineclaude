# MineClaude

A Minecraft clone built entirely by **Claude Opus 4.6** using [Claude Code](https://claude.com/claude-code), Anthropic's agentic coding tool. This project is a test of Claude's ability to build a complex, multi-system game from scratch with minimal human intervention.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/Bevy_0.18-232326?style=flat&logo=bevy&logoColor=white)

## What is this?

MineClaude is a voxel engine and survival game inspired by Minecraft, written in Rust using the Bevy 0.18 game engine. It features terrain generation, caves, mining, crafting, combat, hunger, farming, mobs, and more — all implemented autonomously by Claude.

A human directed the project at a high level ("build a Minecraft clone", "add hunger", "fix this bug") but **wrote zero lines of code**. Every line of Rust, every system, every algorithm was authored by Claude.

## How it was built

The project used a **continuous autonomous development loop** where Claude operated as a team lead coordinating multiple AI agents:

1. **Watchdog agent** scans the codebase for bugs, missing features, and Minecraft inaccuracies
2. **Lead (Claude)** prioritizes findings and creates tasks
3. **Implementer agents** are spawned in parallel to build features, each owning specific modules
4. **Reviewer agent** audits all changes before they're accepted — if issues are found, implementers fix them
5. **Verify** the build compiles, then loop back to step 1

This ran continuously across 12+ development rounds, with the human only stepping in to playtest and report bugs. Claude managed all coordination, task assignment, code review, and conflict resolution between agents.

### Agent team structure

```
Claude (Team Lead / Delegator)
  ├── Watchdog Agent     — finds bugs + suggests features
  ├── Implementer A      — e.g. world generation module
  ├── Implementer B      — e.g. UI module
  ├── Implementer C      — e.g. player mechanics
  └── Reviewer Agent     — mandatory code review gate
```

Each implementer was assigned strict module ownership (specific files only) to prevent conflicts. The lead never wrote code directly — only coordinated.

## Features

- **World generation** — Infinite terrain with plains/desert biomes, noise-based caves (cheese chambers, spaghetti tunnels, noodle passages), ores, trees, tall grass, clay, gravel
- **Mining & building** — Block breaking with tool-appropriate speeds, block placement, correct drops (stone→cobblestone, etc.)
- **Crafting** — 2x2 and 3x3 crafting grids, 40+ recipes
- **Tools & weapons** — Wood/stone/iron/diamond tiers with durability, mining speed multipliers, damage values
- **Armor** — Leather/iron/diamond sets with damage reduction
- **Combat** — Melee attacks, knockback, hostile mobs (zombie/skeleton) with chase AI, night spawning, sunburn
- **Hunger** — Food system with exhaustion from sprinting/jumping/mining, starvation, health regen
- **Farming** — Hoes, seeds, wheat growth stages, bread crafting
- **Survival mechanics** — Fall damage, void damage, drowning, death screen with respawn
- **Mobs** — Cows, sheep, zombies, skeletons with drops (leather, wool, rotten flesh, bones)
- **Storage** — Chests with 27-slot UI, furnace smelting, persistent saves
- **World features** — Doors, beds (spawn point + night skip), torches with point lights, saplings that grow into trees
- **UI** — HUD (health, hunger, armor bars), inventory management, hotbar, item tooltips, debug overlay
- **Rendering** — Custom greedy meshing with WGSL shader, cross-billboard meshes for plants, texture atlas from ProgrammerArt

## Build & Run

Requires Rust (stable) and a system with Vulkan/Metal/DX12 support.

```bash
# Dev build (fast compile)
cargo run --features bevy/dynamic_linking

# Release build (optimized)
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| WASD | Move |
| Space | Jump |
| Ctrl | Sprint |
| Shift | Sneak (edge protection) |
| Left Click | Break block / Attack |
| Right Click | Place block / Interact |
| E | Inventory |
| 1-9 | Hotbar selection |
| Scroll | Cycle hotbar |
| Esc | Pause menu |
| F3 | Debug overlay |

## Tech Stack

- **Language:** Rust
- **Engine:** Bevy 0.18
- **Meshing:** Custom greedy meshing with async chunk generation
- **Physics:** Minecraft-accurate values (gravity, drag, jump velocity)
- **Textures:** ProgrammerArt (CC BY 4.0) packed into a 256x256 atlas at build time
- **Caves:** 3D noise-based (cheese/spaghetti/noodle) inspired by Minecraft 1.18+
- **AI:** State-based mob behavior with pathfinding and day/night cycles

## License

MIT
