# MineClaude Development Workflow

## Philosophy

This is **continuous development**, not one-off sprints. The system runs in a loop:
a **Watchdog agent** finds gaps/bugs → **Lead** prioritizes → **Implementers** fix → **Tester** writes tests → **Reviewer** inspects → **Verify** → repeat.

Lead coordinates, delegates, and verifies. Lead does NOT write code directly.

Key principles:
- Test harness quality is everything — tests must be concise, grep-friendly, and actionable
- Specialization through roles — different agents handle different concerns
- Reference oracle — compare against Minecraft Java Edition behavior
- CI prevents regressions — never merge code that breaks existing functionality
- Context pollution prevention — minimal output, log details to files

## The Continuous Loop

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│   1. DETECT → 2. PRIORITIZE → 3. IMPLEMENT → 4. TEST → 5. REVIEW → 6. VERIFY  │
│       ↑                                                                         │
│       └─────────────────────────────────────────────────────────────────────────┘
```

### 1. DETECT (Watchdog Agent)
A dedicated agent scans for:
- **Feature gaps** vs Minecraft survival (compare current code against Minecraft wiki)
- **Broken mechanics** (wrong block drops, missing recipes, incorrect behavior)
- **Build/test failures** (`cargo build`, `cargo test`, `cargo clippy`)
- **Minecraft accuracy** (break times, recipe correctness, physics values)
- **Code quality** (unwrap(), magic numbers, missing error handling)

The Watchdog reads source code, runs the test suite, and produces a prioritized issue list.
It does NOT fix anything — only reports. Before reporting, it checks what teammates are already working on.

### 2. PRIORITIZE (Lead)
Lead reviews Watchdog findings and picks the highest-impact issues:
- P0 bugs first (crashes, broken core mechanics)
- P0 features next (blockers for survival progression)
- P1 improvements (survival feel, caves, mobs)
- P2 polish last (nice-to-have)

### 3. IMPLEMENT (Implementer Agents)
Lead decomposes chosen work into tasks, spawns teammates, monitors progress.
See [Team Structure](#team-structure) and [Running a Team](#running-a-team) below.

### 4. TEST (Tester Agent)
**After implementers report done**, spawn a **Tester agent** to write tests for all new/changed code:
- Unit tests for pure logic (recipes, inventory ops, block properties, coordinate math)
- Integration tests for multi-module flows (crafting pipeline, smelting pipeline, save/load)
- Edge case tests (empty grids, full inventories, invalid inputs, boundary conditions)
- Regression tests for any bugs fixed in this round
- Run `cargo test` and fix any failures
- The tester reads what implementers changed and writes tests specifically for those changes

### 5. REVIEW (Reviewer Agent)
After tester finishes, a **Reviewer agent** inspects all changes:
- Reads every file modified by implementers AND tester
- Checks for correctness, Minecraft accuracy, code quality
- Verifies cross-module API compatibility (types match, no broken imports)
- Reports issues that need fixing
- If issues found: **send fixes back to implementers** (they're still alive!)
- Only after Reviewer confirms all clear does the round close

### 6. VERIFY (Lead)
- `cargo build` — zero errors
- `cargo test` — zero failures
- Manual spot-check if relevant (rendering, physics)
- Loop back to step 1

## Team Structure

### Roles

| Role | Agent Type | Mode | Description |
|------|-----------|------|-------------|
| **Lead** | — | delegate | Coordinates. Creates tasks, monitors, synthesizes. Never writes code. |
| **Watchdog** | general-purpose | bypassPermissions | Scans codebase for gaps, bugs, Minecraft inaccuracies. Reports only. |
| **Implementer** | general-purpose | bypassPermissions | Writes code within assigned module. Must `cargo build` before done. |
| **Tester** | general-purpose | bypassPermissions | Writes tests for all new/changed code. Must `cargo test` before done. |
| **Reviewer** | code-review-senior | bypassPermissions | Reviews all changes. Must approve before round closes. |
| **Researcher** | Explore or general-purpose | plan | Reads Minecraft wiki, studies algorithms, writes research docs. |

### Team Size Rules
- 3-5 active teammates max (more = coordination overhead exceeds benefit)
- **Multiple implementers in parallel** when tasks touch separate modules/files
- One teammate per module (no file conflicts) — this is why parallelism works
- Watchdog can run as a background subagent between team sprints

## Running a Team

### Task Decomposition
- 4-6 self-contained tasks per round
- Each task: one module, one deliverable, clear done criteria
- Set up dependencies (blockedBy) for cross-module work
- Size tasks for ~15-30 min agent work each

### Spawning
- Create all tasks with `TaskCreate` BEFORE spawning teammates
- Spawn teammates with detailed prompts including:
  - Task context and acceptance criteria
  - Relevant file paths
  - Bevy 0.18 API notes (see CLAUDE.md and memory)
  - Module ownership boundaries
  - Reference to research docs if applicable

### Monitoring
- Check task list regularly
- Redirect teammates hitting dead ends
- Verify cross-module API compatibility ASAP (type mismatches are common)
- If a teammate is stuck for >2 messages, intervene with guidance

### Agent Lifecycle — CRITICAL RULES

**NEVER shut down implementers early.** The full lifecycle is:

```
1. Implementers work on tasks → report done → stay IDLE (alive)
2. Tester writes tests for implementer changes → report done → stays IDLE
3. Reviewer inspects everything → reports issues OR approves
4. IF issues found → send fixes back to idle implementers → they fix → back to step 3
5. ONLY when Reviewer approves with no issues → shut down everyone
```

Why: Implementers have full context about their changes. If the reviewer or tester finds issues, the implementers can fix them instantly without re-spawning and re-reading everything. Keeping them idle costs nothing — they only consume resources when active.

**Shutdown order (only after final approval):**
1. Implementers (they have nothing left to fix)
2. Tester
3. Reviewer
4. Watchdog (persists across cycles, only at end of session)
5. Clean up team resources

## Watchdog Agent Prompt Template

When spawning a Watchdog, use this prompt structure:

```
You are the Watchdog for MineClaude, an incomplete Minecraft clone in Rust + Bevy 0.18.
You have THREE responsibilities:

## PART 1: QUALITY — Find bugs and accuracy issues in existing code
Scan the codebase for:
- Incorrect block drops, break times, crafting outputs vs vanilla Minecraft
- Physics inaccuracies (speeds, jump height, gravity, fall damage)
- Code quality (unwrap(), panics, magic numbers, duplicated logic)
- Cross-module type mismatches or broken imports
- Performance concerns (O(n^2) loops, excessive allocations)

## PART 2: ROADMAP — Chart the path to a complete survival experience
Think like a game designer. Walk through the Minecraft survival progression:
  Spawn → punch tree → craft planks → craft tools → mine stone → build shelter →
  survive night → mine ores → smelt iron → explore caves → find diamonds → enchant → beat game

For EACH step, ask: "Can the player do this in MineClaude right now?"
Where the chain breaks, that's what needs building next.

Consider ALL dimensions of the experience:
- **Blocks**: doors, beds, chests, ladders, fences, stairs, slabs, TNT, pistons
- **Items**: armor, bow+arrows, buckets, compass, maps, books, beds
- **Mobs**: creeper, spider, enderman, villagers, animals (pig, chicken)
- **World**: villages, dungeons, mineshafts, strongholds, nether portal
- **Mechanics**: hunger, XP, enchanting, brewing, redstone, farming, breeding
- **UI**: durability bars, XP bar, armor slots, creative inventory
- **Audio**: mob sounds, ambient music, cave ambiance, weather sounds
- **Gameplay loops**: farming, animal breeding, fishing, trading, exploration

Prioritize by what unblocks the most gameplay. A chest matters more than pistons.
A bed (skip night) matters more than enchanting.

## PART 3: COORDINATE — Check what's already being worked on
Before finalizing your report:
1. Read the task list (use TaskList tool) to see what tasks are in progress
2. Send a message to active teammates asking what they're currently working on
3. In your report, mark any issue that overlaps with in-progress work as:
   IN_PROGRESS: [teammate] [description of what they're fixing]
4. Do NOT re-report bugs that teammates are actively fixing — just confirm they're covered
5. Focus your NEW findings on gaps that nobody is addressing yet

## OUTPUT FORMAT (one line per item, grep-friendly):
BUG: [module] [description]
ACCURACY: [module] [expected] vs [actual]
QUALITY: [file:line] [description]
SUGGEST: [high] [feature] — [why it matters for survival progression]
SUGGEST: [medium] [feature] — [what it adds to the experience]
SUGGEST: [low] [feature] — [nice to have, not blocking]

All sections are equally important. Spend equal effort on each.
Do NOT fix anything. Report and suggest.
```

## Tester Agent Prompt Template

When spawning a Tester, use this prompt structure:

```
You are the Tester for MineClaude, a Minecraft clone in Rust + Bevy 0.18.

Your job: write comprehensive tests for all code changed in this round.
The following implementers made changes: [LIST CHANGES HERE]

## What to test:
1. **Unit tests** — Pure logic that doesn't need Bevy runtime:
   - Block properties (is_solid, break_time, drop_type, display_name)
   - Inventory operations (add_item, remove_item, slot swaps)
   - Crafting recipe matching (2x2 and 3x3, all recipes, edge cases)
   - Smelting logic (input→output, fuel values)
   - Coordinate math (world_to_chunk, world_to_local)
   - Tool properties (durability, speed, damage, tier requirements)

2. **Edge cases** — Things that commonly break:
   - Empty grids, full inventories, stack overflow
   - Invalid/unexpected inputs
   - Boundary conditions (Y=0, Y=255, chunk borders)
   - Items that shouldn't stack (tools)

3. **Regression tests** — For any bugs fixed this round

## Where to put tests:
- In-module tests: `#[cfg(test)] mod tests { ... }` at bottom of the relevant .rs file
- Integration tests: `tests/` directory (create if needed)

## Rules:
- Every test must have a descriptive name: `test_stone_pickaxe_breaks_stone_faster_than_hand`
- Use assert_eq! with clear expected values
- Test BOTH positive cases (should work) AND negative cases (should NOT work)
- Run `cargo test` after writing tests — all must pass
- If a test reveals a bug, report it but don't fix it — that's the implementer's job

## Output:
Report what tests you wrote, what passed, and any bugs discovered.
```

## Known Minecraft Mechanics to Verify

### Block Drops (common mistakes)
- Grass block → drops Dirt (not grass)
- Stone → drops Cobblestone (not stone, unless Silk Touch)
- Coal ore → drops Coal item (not the ore block, unless Silk Touch)
- Diamond ore → drops Diamond item
- Gravel → drops Gravel (10% chance of flint)
- Glass → drops nothing (unless Silk Touch)
- Leaves → small chance of sapling + apple
- Tall grass → small chance of seeds

### Tool Requirements
- Wood/dirt/sand: breakable by hand, faster with shovel
- Stone/cobblestone/ore: requires pickaxe to drop items
- Iron ore: requires stone+ pickaxe
- Diamond ore: requires iron+ pickaxe
- Gold ore: requires iron+ pickaxe
- Obsidian: requires diamond pickaxe

### Crafting Grid Sizes
- Player inventory: 2x2 grid (planks, sticks, basic items)
- Crafting table: 3x3 grid (tools, weapons, armor, complex items)

## Testing Strategy

### Unit Tests (in-module `#[cfg(test)]`)
Pure logic that doesn't need Bevy runtime:
- `block/mod.rs`: BlockType methods (is_solid, break_time, drop_type, display_name)
- `inventory/inventory.rs`: add_item, remove_item, slot operations
- `inventory/crafting.rs`: recipe matching, output calculation — **test ALL recipes, ALL grid positions**
- `inventory/item.rs`: tool properties, stack sizes, durability values
- `world/coordinates.rs`: world_to_chunk_pos, world_to_local_pos
- `world/chunk.rs`: get/set, is_empty
- `player/interaction.rs`: tool speed multipliers, weapon damage values

### Integration Tests (tests/ directory)
Multiple modules, no rendering:
- World generation: generate chunk, verify block distribution
- Save/load: write chunk, read back, compare
- Crafting: full recipe → output pipeline

### Test Output Format
```
OK: [module] [description]
FAIL: [module] [description] — [reason]
```

### Regression Prevention
- `cargo build` must pass at all times
- `cargo test` must pass before any task is marked complete
- New features must include at least one test

## Quality Gates

Before marking any round of work complete:
1. `cargo build` — zero errors
2. `cargo test` — zero failures
3. No `unwrap()` in production code
4. All new constants use `pub const`
5. Reviewer approves all changes

## Communication Protocol

### Lead → Teammate
- Clear task with file paths and acceptance criteria
- Bevy 0.18 API warnings if relevant
- Module ownership boundaries

### Teammate → Lead
- "Task complete" with summary of changes
- "Blocked on X" if waiting on another module
- "Build fails because Y" with first error only

### Between Teammates
- Only for cross-module API contracts
- Share type definitions, function signatures only
- Keep messages short

## Minecraft Reference URLs

- Break times & drops: https://minecraft.wiki/w/Breaking
- Crafting recipes: https://minecraft.wiki/w/Crafting
- Tool tiers: https://minecraft.wiki/w/Tiers
- Mob stats: https://minecraft.wiki/w/Mob
- Cave generation: See `research/cave_generation.md`
