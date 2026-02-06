use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin, SuperSimplex};
use std::sync::RwLock;
use super::chunk::{Chunk, CHUNK_SIZE};
use crate::block::BlockType;

const DEFAULT_SEED: u32 = 42;
const SEA_LEVEL: i32 = 63;

/// Terrain height range
const BASE_HEIGHT: f64 = 64.0;
const HEIGHT_AMPLITUDE: f64 = 30.0;

/// Noise scales
const TERRAIN_FREQUENCY: f64 = 0.005;
const BIOME_FREQUENCY: f64 = 0.002;

/// Cave thresholds
pub const CHEESE_THRESHOLD: f64 = 0.45;
pub const SPAGHETTI_THRESHOLD: f64 = 0.15;
pub const NOODLE_THRESHOLD: f64 = 0.08;

/// Resettable terrain noise instance — can be re-initialized with a new seed.
static TERRAIN_NOISE: RwLock<Option<TerrainNoise>> = RwLock::new(None);

/// Set the world seed and reinitialize all terrain noise generators.
pub fn set_world_seed(seed: u32) {
    let noise = TerrainNoise::with_seed(seed);
    let mut guard = TERRAIN_NOISE.write().expect("TERRAIN_NOISE lock poisoned");
    *guard = Some(noise);
}

/// Get a reference to the terrain noise, initializing with default seed if needed.
fn with_noise<F, R>(f: F) -> R
where
    F: FnOnce(&TerrainNoise) -> R,
{
    {
        let guard = TERRAIN_NOISE.read().expect("TERRAIN_NOISE lock poisoned");
        if let Some(ref noise) = *guard {
            return f(noise);
        }
    }
    // Initialize with default seed
    set_world_seed(DEFAULT_SEED);
    let guard = TERRAIN_NOISE.read().expect("TERRAIN_NOISE lock poisoned");
    f(guard.as_ref().expect("just initialized"))
}

/// Pre-computed noise generators for terrain generation.
struct TerrainNoise {
    height: Fbm<Perlin>,
    biome_temp: SuperSimplex,
    /// Cheese caves — large chambers (Fbm for interesting shapes)
    cave_cheese: Fbm<Perlin>,
    /// Spaghetti caves — winding tunnels (two Perlin for zero-crossing intersection)
    cave_spaghetti_a: Perlin,
    cave_spaghetti_b: Perlin,
    /// Noodle caves — thin squiggly passages (two Perlin, higher frequency)
    cave_noodle_a: Perlin,
    cave_noodle_b: Perlin,
    /// Pre-computed ore noise generators (diamond=0, gold=1, iron=2, coal=3)
    ore_noises: [Perlin; 4],
    /// General-purpose ore noise for tree placement etc.
    ore: Perlin,
    /// Gravel patch noise for underground deposits
    gravel: Perlin,
    /// Clay patch noise for water-adjacent deposits
    clay: Perlin,
    /// Tall grass scatter noise
    grass_scatter: Perlin,
}

// SAFETY: Perlin, Fbm<Perlin>, SuperSimplex are all deterministic and read-only after creation.
unsafe impl Send for TerrainNoise {}
unsafe impl Sync for TerrainNoise {}

impl TerrainNoise {
    fn with_seed(seed: u32) -> Self {
        let height = Fbm::<Perlin>::new(seed)
            .set_octaves(3)
            .set_frequency(TERRAIN_FREQUENCY)
            .set_persistence(0.5)
            .set_lacunarity(2.0);

        let biome_temp = SuperSimplex::new(seed.wrapping_add(1));

        let cave_cheese = Fbm::<Perlin>::new(seed.wrapping_add(10))
            .set_octaves(2)
            .set_frequency(0.02);

        let cave_spaghetti_a = Perlin::new(seed.wrapping_add(20));
        let cave_spaghetti_b = Perlin::new(seed.wrapping_add(21));

        let cave_noodle_a = Perlin::new(seed.wrapping_add(30));
        let cave_noodle_b = Perlin::new(seed.wrapping_add(31));

        let ore = Perlin::new(seed.wrapping_add(3));

        let ore_noises = [
            Perlin::new(seed.wrapping_add(50)),
            Perlin::new(seed.wrapping_add(51)),
            Perlin::new(seed.wrapping_add(52)),
            Perlin::new(seed.wrapping_add(53)),
        ];

        let gravel = Perlin::new(seed.wrapping_add(40));
        let clay = Perlin::new(seed.wrapping_add(41));
        let grass_scatter = Perlin::new(seed.wrapping_add(42));

        Self {
            height,
            biome_temp,
            cave_cheese,
            cave_spaghetti_a,
            cave_spaghetti_b,
            cave_noodle_a,
            cave_noodle_b,
            ore_noises,
            ore,
            gravel,
            clay,
            grass_scatter,
        }
    }

    /// Sample terrain height at a world (x, z) position.
    fn sample_height(&self, world_x: i32, world_z: i32) -> i32 {
        let val = self.height.get([world_x as f64, world_z as f64]);
        // Fbm output roughly in [-1, 1]; map to height range
        (BASE_HEIGHT + val * HEIGHT_AMPLITUDE) as i32
    }

    /// Returns a temperature value in roughly [-1, 1].
    /// Positive = warm (desert), negative = cool (plains).
    fn sample_biome_temp(&self, world_x: i32, world_z: i32) -> f64 {
        self.biome_temp
            .get([world_x as f64 * BIOME_FREQUENCY, world_z as f64 * BIOME_FREQUENCY])
    }

    /// Returns true if this position should be carved out as a cave.
    /// Uses three cave types: cheese (chambers), spaghetti (tunnels), noodle (thin passages).
    fn is_cave(&self, wx: i32, wy: i32, wz: i32, terrain_height: i32) -> bool {
        // No caves at bedrock
        if wy <= 0 {
            return false;
        }
        let (x, y, z) = (wx as f64, wy as f64, wz as f64);

        // Cheese caves — large chambers, keep 4-block surface protection (giant holes look ugly)
        if wy <= terrain_height - 4 {
            let cheese = self.cave_cheese.get([x, y, z]);
            if cheese > CHEESE_THRESHOLD {
                return true;
            }
        }

        // Spaghetti caves — winding tunnels, no surface protection (natural entrances)
        if wy <= terrain_height {
            let spa_a = self.cave_spaghetti_a.get([x * 0.04, y * 0.04, z * 0.04]);
            let spa_b = self.cave_spaghetti_b.get([x * 0.04, y * 0.04, z * 0.04]);
            if spa_a.abs() + spa_b.abs() < SPAGHETTI_THRESHOLD {
                return true;
            }
        }

        // Noodle caves — thin squiggly passages, no surface protection (small openings)
        if wy <= terrain_height {
            let noo_a = self.cave_noodle_a.get([x * 0.08, y * 0.08, z * 0.08]);
            let noo_b = self.cave_noodle_b.get([x * 0.08, y * 0.08, z * 0.08]);
            if noo_a.abs() + noo_b.abs() < NOODLE_THRESHOLD {
                return true;
            }
        }

        false
    }

    /// Returns true if gravel should replace stone at this underground position.
    fn is_gravel(&self, wx: i32, wy: i32, wz: i32) -> bool {
        let val = self.gravel.get([
            wx as f64 * 0.05,
            wy as f64 * 0.05,
            wz as f64 * 0.05,
        ]);
        val > 0.7
    }

    /// Returns true if clay should be placed at this near-water position.
    fn is_clay(&self, wx: i32, wz: i32) -> bool {
        let val = self.clay.get([wx as f64 * 0.08, wz as f64 * 0.08]);
        val > 0.5
    }

    /// Returns a scatter value for tall grass placement.
    fn grass_noise(&self, wx: i32, wz: i32) -> f64 {
        self.grass_scatter.get([wx as f64 * 0.3, wz as f64 * 0.3])
    }

    /// Ore noise sample for a given position and ore-specific index (0-3).
    fn ore_density(&self, wx: i32, wy: i32, wz: i32, index: usize) -> f64 {
        self.ore_noises[index].get([
            wx as f64 * 0.1,
            wy as f64 * 0.1,
            wz as f64 * 0.1,
        ])
    }
}

#[derive(Clone, Copy)]
enum Biome {
    Plains,
    Desert,
}

/// Triangular distribution probability: peaks at `peak`, zero at `min` and `max`.
fn triangular_weight(y: i32, min: i32, max: i32, peak: i32) -> f64 {
    if y < min || y > max {
        return 0.0;
    }
    let y = y as f64;
    let min = min as f64;
    let max = max as f64;
    let peak = peak as f64;
    if y <= peak {
        (y - min) / (peak - min)
    } else {
        (max - y) / (max - peak)
    }
}

/// Determine which ore (if any) should replace stone at this position.
fn determine_ore(noise: &TerrainNoise, wx: i32, wy: i32, wz: i32) -> Option<BlockType> {
    // Diamond: Y 5-16, peak 8 (index 0)
    let diamond_weight = triangular_weight(wy, 5, 16, 8);
    if diamond_weight > 0.0 {
        let density = noise.ore_density(wx, wy, wz, 0);
        if density > 1.0 - diamond_weight * 0.35 {
            return Some(BlockType::DiamondOre);
        }
    }

    // Gold: Y 5-30, peak 12 (index 1)
    let gold_weight = triangular_weight(wy, 5, 30, 12);
    if gold_weight > 0.0 {
        let density = noise.ore_density(wx, wy, wz, 1);
        if density > 1.0 - gold_weight * 0.45 {
            return Some(BlockType::GoldOre);
        }
    }

    // Iron: Y 5-54, peak 28 (index 2)
    let iron_weight = triangular_weight(wy, 5, 54, 28);
    if iron_weight > 0.0 {
        let density = noise.ore_density(wx, wy, wz, 2);
        if density > 1.0 - iron_weight * 0.68 {
            return Some(BlockType::IronOre);
        }
    }

    // Coal: Y 5-95, peak 48 (index 3)
    let coal_weight = triangular_weight(wy, 5, 95, 48);
    if coal_weight > 0.0 {
        let density = noise.ore_density(wx, wy, wz, 3);
        if density > 1.0 - coal_weight * 0.82 {
            return Some(BlockType::CoalOre);
        }
    }

    None
}

/// Generate a complete chunk at the given chunk position.
pub fn generate_chunk(chunk_pos: IVec3) -> Chunk {
    with_noise(|noise| generate_chunk_with_noise(chunk_pos, noise))
}

/// Sample terrain height at a world (x, z) position without generating a full chunk.
pub fn sample_terrain_height(world_x: i32, world_z: i32) -> i32 {
    with_noise(|noise| noise.sample_height(world_x, world_z))
}

fn generate_chunk_with_noise(chunk_pos: IVec3, noise: &TerrainNoise) -> Chunk {
    let mut chunk = Chunk::default();

    let world_x_base = chunk_pos.x * CHUNK_SIZE as i32;
    let world_y_base = chunk_pos.y * CHUNK_SIZE as i32;
    let world_z_base = chunk_pos.z * CHUNK_SIZE as i32;

    // Pre-compute height map and biome for this chunk's XZ columns
    let mut height_map = [[0i32; CHUNK_SIZE]; CHUNK_SIZE];
    let mut biome_map = [[Biome::Plains; CHUNK_SIZE]; CHUNK_SIZE];

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let wx = world_x_base + x as i32;
            let wz = world_z_base + z as i32;
            height_map[z][x] = noise.sample_height(wx, wz);

            let temp = noise.sample_biome_temp(wx, wz);
            biome_map[z][x] = if temp > 0.3 {
                Biome::Desert
            } else {
                Biome::Plains
            };
        }
    }

    // First pass: fill terrain
    for y in 0..CHUNK_SIZE {
        let wy = world_y_base + y as i32;
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let terrain_height = height_map[z][x];
                let biome = biome_map[z][x];

                let block = if wy < 0 {
                    BlockType::Air
                } else if wy == 0 {
                    BlockType::Bedrock
                } else if wy > terrain_height {
                    // Above terrain: water if below sea level, else air
                    if wy <= SEA_LEVEL {
                        BlockType::Water
                    } else {
                        BlockType::Air
                    }
                } else if wy == terrain_height && terrain_height >= SEA_LEVEL {
                    // Surface block (only when above water)
                    match biome {
                        Biome::Plains => BlockType::Grass,
                        Biome::Desert => BlockType::Sand,
                    }
                } else if wy > terrain_height - 4 && wy < terrain_height {
                    // Sub-surface layers (3-4 blocks deep)
                    match biome {
                        Biome::Plains => BlockType::Dirt,
                        Biome::Desert => BlockType::Sandstone,
                    }
                } else if wy == terrain_height && terrain_height < SEA_LEVEL {
                    // Underwater surface
                    match biome {
                        Biome::Plains => BlockType::Dirt,
                        Biome::Desert => BlockType::Sand,
                    }
                } else {
                    // Deep underground: stone (with ore/cave pass later)
                    BlockType::Stone
                };

                chunk.set(x, y, z, block);
            }
        }
    }

    // Second pass: carve caves and place ores
    for y in 0..CHUNK_SIZE {
        let wy = world_y_base + y as i32;
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let wx = world_x_base + x as i32;
                let wz = world_z_base + z as i32;
                let current = chunk.get(x, y, z);

                // Cave carving — carves through stone, dirt, sand, grass, gravel, sandstone
                let terrain_height = height_map[z][x];
                let is_carveable = matches!(
                    current,
                    BlockType::Stone
                        | BlockType::Dirt
                        | BlockType::Grass
                        | BlockType::Sand
                        | BlockType::Sandstone
                        | BlockType::Gravel
                );
                if is_carveable && noise.is_cave(wx, wy, wz, terrain_height) {
                    chunk.set(x, y, z, BlockType::Air);
                    continue;
                }

                if current == BlockType::Stone {
                    // Gravel patches underground (below Y=60)
                    if wy < 60 && noise.is_gravel(wx, wy, wz) {
                        chunk.set(x, y, z, BlockType::Gravel);
                        continue;
                    }

                    // Ore placement
                    if let Some(ore) = determine_ore(noise, wx, wy, wz) {
                        chunk.set(x, y, z, ore);
                    }
                }

                // Clay patches near water (Y 60-63, replace sand/dirt)
                if (current == BlockType::Sand || current == BlockType::Dirt)
                    && wy >= 60
                    && wy <= SEA_LEVEL
                    && noise.is_clay(wx, wz)
                {
                    chunk.set(x, y, z, BlockType::Clay);
                }
            }
        }
    }

    // Third pass: place trees and vegetation (only if this chunk contains the surface)
    place_trees(&mut chunk, noise, &height_map, &biome_map, chunk_pos);
    place_tall_grass(&mut chunk, noise, &height_map, &biome_map, chunk_pos);

    chunk
}

/// Place trees on grass blocks in plains biome.
fn place_trees(
    chunk: &mut Chunk,
    noise: &TerrainNoise,
    height_map: &[[i32; CHUNK_SIZE]; CHUNK_SIZE],
    biome_map: &[[Biome; CHUNK_SIZE]; CHUNK_SIZE],
    chunk_pos: IVec3,
) {
    let world_x_base = chunk_pos.x * CHUNK_SIZE as i32;
    let world_y_base = chunk_pos.y * CHUNK_SIZE as i32;
    let world_z_base = chunk_pos.z * CHUNK_SIZE as i32;

    // Use a simple hash-based check for tree placement to ensure spacing
    for z in 2..CHUNK_SIZE - 2 {
        for x in 2..CHUNK_SIZE - 2 {
            let wx = world_x_base + x as i32;
            let wz = world_z_base + z as i32;
            let terrain_height = height_map[z][x];

            // Only place trees on plains grass above sea level
            if !matches!(biome_map[z][x], Biome::Plains) {
                continue;
            }
            if terrain_height < SEA_LEVEL {
                continue;
            }

            // Use noise-based spacing: sample at grid-snapped positions
            let grid_x = wx.div_euclid(7);
            let grid_z = wz.div_euclid(7);
            let tree_noise = noise.ore.get([grid_x as f64 * 1.5, grid_z as f64 * 1.5]);
            if tree_noise < 0.2 {
                continue;
            }

            // Only place tree if this is the "chosen" block within its grid cell
            let cell_x = wx.rem_euclid(7);
            let cell_z = wz.rem_euclid(7);
            if cell_x != 3 || cell_z != 3 {
                continue;
            }

            // Surface must be in this chunk
            let local_surface_y = terrain_height - world_y_base;
            if local_surface_y < 0 || local_surface_y >= CHUNK_SIZE as i32 {
                continue;
            }

            // Verify the surface block is actually grass
            if chunk.get(x, local_surface_y as usize, z) != BlockType::Grass {
                continue;
            }

            // Determine tree type: ~30% birch, ~70% oak
            // Use a secondary noise sample for deterministic variety
            let variety_noise = noise.ore.get([wx as f64 * 0.7, wz as f64 * 0.7]);
            let is_birch = variety_noise > 0.4; // roughly 30% of range maps to birch

            let (log_type, leaf_type, trunk_height) = if is_birch {
                // Birch: taller trunk (5-7), thinner canopy
                let h = 5 + ((tree_noise * 10.0) as i32 % 3); // 5, 6, or 7
                (BlockType::BirchLog, BlockType::BirchLeaves, h)
            } else {
                // Oak: 5-6 blocks trunk
                let h = 5 + ((tree_noise * 10.0) as i32 % 2); // 5 or 6
                (BlockType::OakLog, BlockType::OakLeaves, h)
            };

            // Place trunk
            for ty in 1..=trunk_height {
                let ly = local_surface_y + ty;
                if ly >= 0 && ly < CHUNK_SIZE as i32 {
                    chunk.set(x, ly as usize, z, log_type);
                }
            }

            // Place leaves: birch has narrower canopy (radius 1), oak has radius 2
            let leaf_start = trunk_height - 2;
            let max_radius: i32 = if is_birch { 1 } else { 2 };
            for ly_offset in leaf_start..=trunk_height + 1 {
                let ly = local_surface_y + ly_offset;
                if ly < 0 || ly >= CHUNK_SIZE as i32 {
                    continue;
                }
                // Narrower at top layers
                let radius: i32 = if ly_offset >= trunk_height {
                    1.min(max_radius)
                } else {
                    max_radius
                };
                for dz in -radius..=radius {
                    for dx in -radius..=radius {
                        // Skip corners for rounder shape
                        if dx.abs() == radius && dz.abs() == radius {
                            continue;
                        }
                        let lx = x as i32 + dx;
                        let lz = z as i32 + dz;
                        if lx < 0
                            || lx >= CHUNK_SIZE as i32
                            || lz < 0
                            || lz >= CHUNK_SIZE as i32
                        {
                            continue;
                        }
                        // Don't overwrite trunk
                        if dx == 0 && dz == 0 && ly_offset <= trunk_height {
                            continue;
                        }
                        let current = chunk.get(lx as usize, ly as usize, lz as usize);
                        if current == BlockType::Air {
                            chunk.set(lx as usize, ly as usize, lz as usize, leaf_type);
                        }
                    }
                }
            }
        }
    }
}

/// Place tall grass on exposed grass blocks in plains biome (~20% coverage).
fn place_tall_grass(
    chunk: &mut Chunk,
    noise: &TerrainNoise,
    height_map: &[[i32; CHUNK_SIZE]; CHUNK_SIZE],
    biome_map: &[[Biome; CHUNK_SIZE]; CHUNK_SIZE],
    chunk_pos: IVec3,
) {
    let world_x_base = chunk_pos.x * CHUNK_SIZE as i32;
    let world_y_base = chunk_pos.y * CHUNK_SIZE as i32;
    let world_z_base = chunk_pos.z * CHUNK_SIZE as i32;

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            if !matches!(biome_map[z][x], Biome::Plains) {
                continue;
            }

            let terrain_height = height_map[z][x];
            if terrain_height < SEA_LEVEL {
                continue;
            }

            // The block above the surface must be in this chunk
            let local_above = terrain_height + 1 - world_y_base;
            if local_above < 0 || local_above >= CHUNK_SIZE as i32 {
                continue;
            }

            let local_surface = terrain_height - world_y_base;
            if local_surface < 0 || local_surface >= CHUNK_SIZE as i32 {
                continue;
            }

            // Only place on grass blocks that have air above
            if chunk.get(x, local_surface as usize, z) != BlockType::Grass {
                continue;
            }
            if chunk.get(x, local_above as usize, z) != BlockType::Air {
                continue;
            }

            let wx = world_x_base + x as i32;
            let wz = world_z_base + z as i32;
            let val = noise.grass_noise(wx, wz);
            // ~20% coverage: noise range is roughly [-1, 1], so > 0.6 gives ~20%
            if val > 0.6 {
                chunk.set(x, local_above as usize, z, BlockType::TallGrass);
            }
        }
    }
}
