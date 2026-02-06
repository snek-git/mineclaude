# MineClaude

A Minecraft clone built entirely by **Claude Opus 4.6** using [Claude Code](https://claude.com/claude-code), Anthropic's agentic coding tool. This project is a test of Claude's ability to build a complex, multi-system game from scratch.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/Bevy_0.18-232326?style=flat&logo=bevy&logoColor=white)

## What is this?

MineClaude is a voxel engine and survival game inspired by Minecraft, written in Rust using the Bevy 0.18 game engine. ~14,000 lines of Rust across 41 source files — terrain generation, caves, mining, crafting, combat, hunger, farming, mobs, and more.

## How it was built

This project was built using Claude Code's **agent teams** feature — multiple Claude agents running in parallel, coordinating through a shared task list and message passing. One lead agent (never writing code itself) orchestrated the entire process, spawning and managing specialized teammate agents.

The full workflow documentation is in [`.claude/WORKFLOW.md`](.claude/WORKFLOW.md), and the project conventions are in [`.claude/CLAUDE.md`](.claude/CLAUDE.md).

### Agent teams

Claude Code's agent teams allow a lead agent to spawn multiple sub-agents that work concurrently. Each agent runs as an independent Claude Code instance with its own context, tools, and file access. They communicate via direct messages and coordinate through a shared task board.

The team structure for each development round:

```
Lead (Delegator — coordinates only, never writes code)
  ├── Watchdog        — scans codebase for bugs, gaps, and Minecraft inaccuracies
  ├── Implementer A   — owns specific source files (e.g. world generation)
  ├── Implementer B   — owns different files (e.g. UI module)
  ├── Implementer C   — owns different files (e.g. player mechanics)
  ├── Tester          — writes tests for all changed code
  └── Reviewer        — mandatory code review gate before changes are accepted
```

Key design decisions that made this work:
- **Strict module ownership** — each implementer was assigned specific files and could only edit those, preventing merge conflicts between parallel agents
- **Implementers stay alive through review** — agents aren't shut down after finishing their task; they stay idle so the reviewer can send fixes back without re-spawning and losing context
- **Shared task board** — all agents read from the same task list, claim work, and mark completion. The lead monitors progress and redirects stuck agents
- **Message-based coordination** — agents communicate through direct messages for cross-module API contracts (e.g. "I added a new BlockType variant, here's the signature")

### The development loop

```
DETECT → PRIORITIZE → IMPLEMENT → TEST → REVIEW → VERIFY → repeat
```

1. **Watchdog agent** continuously scans the codebase for bugs, missing features, and Minecraft inaccuracies
2. **Lead agent** prioritizes findings and decomposes them into tasks
3. **Implementer agents** are spawned in parallel, each assigned strict module ownership
4. **Tester agent** writes tests for all new/changed code
5. **Reviewer agent** audits all changes — if issues are found, they're sent back to the (still-alive) implementers for fixes
6. **Verify** the build compiles and tests pass, then loop back to step 1

This ran across 12+ development rounds. The lead managed all coordination, task assignment, code review, and conflict resolution between agents.

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
- **AI:** State-based mob behavior with day/night cycles

## License

MIT
