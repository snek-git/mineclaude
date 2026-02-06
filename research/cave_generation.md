# Modern Minecraft Cave Generation (1.18+)

Research notes for implementing improved cave generation in MineClaude.

## Overview

Minecraft 1.18 replaced the old "carver" cave system with noise-based caves that use 3D density
functions. Three cave types combine to create varied underground spaces:

1. **Cheese caves** — Large open chambers with pillars
2. **Spaghetti caves** — Long winding tunnels (classic feel)
3. **Noodle caves** — Thin squiggly passages (1-5 blocks wide)

All three use the same principle: evaluate a 3D noise function, carve air where the value
crosses a threshold.

## How Each Cave Type Works

### Cheese Caves
- Use a 3D Perlin noise field
- Where noise value > threshold → air (creates "holes in cheese")
- The `hollowness` parameter controls chamber size
- Noise pillars generate inside large chambers for structural variety
- These create the dramatic open caverns

**Implementation approach:**
```
noise_value = perlin_3d(x * freq, y * freq, z * freq)
if noise_value > cheese_threshold:
    block = Air
```

### Spaghetti Caves
- Use TWO 3D noise fields
- Cave forms at the **intersection/edge** between high and low regions of each noise
- Specifically: where `abs(noise1) + abs(noise2)` is small (near zero-crossings)
- This creates long continuous tunnels that wind through space
- The `thickness` parameter controls tunnel width

**Implementation approach:**
```
n1 = perlin_3d(x * freq1, y * freq1, z * freq1)
n2 = perlin_3d(x * freq2, y * freq2, z * freq2)
spaghetti = abs(n1) + abs(n2)
if spaghetti < spaghetti_threshold:
    block = Air
```

### Noodle Caves
- Same principle as spaghetti but with higher frequency noise and tighter threshold
- Creates passages 1-5 blocks wide
- Uses `noodle_thickness` noise to vary width
- Uses `noodle_ridge_a` and `noodle_ridge_b` noises for shape modulation

**Implementation approach:**
```
n1 = perlin_3d(x * freq_high, y * freq_high, z * freq_high)
n2 = perlin_3d(x * freq_high2, y * freq_high2, z * freq_high2)
noodle = abs(n1) + abs(n2)
thickness_mod = perlin_3d(x * 0.01, y * 0.01, z * 0.01) * 0.05
if noodle < (noodle_threshold + thickness_mod):
    block = Air
```

## Combined Density Function

The final cave determination combines all three types:

```
is_cave = cheese_test(x,y,z) OR spaghetti_test(x,y,z) OR noodle_test(x,y,z)
```

With additional constraints:
- No caves in top 4 blocks of terrain (surface protection)
- No caves below bedrock layer
- Caves below sea level fill with water (aquifers in vanilla, simpler water fill for us)
- Depth-based bias: caves are more common at certain Y levels

## Recommended Parameters for MineClaude

Based on Minecraft behavior and our 16x16x16 chunk system:

### Cheese Caves
- Noise: Perlin, seed offset +10
- Frequency: 0.02 (large-scale features)
- Threshold: 0.6 (higher = fewer, smaller caves)
- Y range: 5 to terrain_height - 8
- Octaves: 2 (Fbm for more interesting shapes)

### Spaghetti Caves
- Noise A: Perlin, seed offset +20
- Noise B: Perlin, seed offset +21
- Frequency: 0.04 (medium tunnels)
- Threshold: 0.06 (how close to zero-crossing = how wide)
- Y range: 5 to terrain_height - 4
- No Fbm needed (single octave is fine for tunnels)

### Noodle Caves
- Noise A: Perlin, seed offset +30
- Noise B: Perlin, seed offset +31
- Frequency: 0.08 (higher freq = thinner, more wiggly)
- Threshold: 0.03 (very tight = very narrow)
- Y range: 5 to terrain_height - 4

### Surface Protection
- Don't carve within 4 blocks of terrain surface
- Reduces ugly surface holes while keeping caves connected underground

### Water Fill
- Caves below sea level (Y=63) fill with water instead of air
- Matches current behavior

## Implementation Plan

Replace the current single-noise cave system in `src/world/generation.rs`:

1. Add new noise generators to `TerrainNoise`:
   - `cave_cheese: Fbm<Perlin>` (2 octaves, freq 0.02)
   - `cave_spaghetti_a: Perlin` (freq 0.04)
   - `cave_spaghetti_b: Perlin` (freq 0.04, different seed)
   - `cave_noodle_a: Perlin` (freq 0.08)
   - `cave_noodle_b: Perlin` (freq 0.08, different seed)

2. Replace `is_cave()` method with `cave_density()` that checks all three types

3. Add surface protection: pass terrain_height to cave check, skip if within 4 blocks

4. Keep the water-fill-below-sea-level behavior

## Sources

- [Minecraft Wiki: World Generation](https://minecraft.wiki/w/World_generation)
- [Minecraft Wiki: Noise Router](https://minecraft.wiki/w/Noise_router)
- [Density functions in 22w07a (misode)](https://gist.github.com/misode/8fa66c37bd8a468d5090327c8acc519e)
- [Alan Zucconi: The World Generation of Minecraft](https://www.alanzucconi.com/2022/06/05/minecraft-world-generation/)
