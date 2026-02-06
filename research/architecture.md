# Rust Voxel Engine Architecture Patterns

Comprehensive research on architecture patterns for building a Minecraft-like voxel engine in Rust with Bevy.

---

## 1. Chunk Data Structures

A chunk is a fixed-size 3D volume of voxels, typically 16x16x16 or 32x32x32 blocks. The choice of internal data structure has massive implications for memory usage, cache performance, and meshing speed.

### Flat Array (Recommended for Minecraft-like)

The simplest and most cache-friendly approach. A 1D array indexed by `(x, y, z)` position within the chunk.

```rust
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE; // 4096

#[derive(Clone)]
pub struct Chunk {
    blocks: [BlockId; CHUNK_VOLUME],
}

impl Chunk {
    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        // YZX ordering (matches Minecraft Anvil) - better compression for terrain
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        self.blocks[Self::index(x, y, z)] = block;
    }
}
```

**Pros**: O(1) access, excellent cache locality, trivial to implement, fastest meshing iteration.
**Cons**: Fixed memory cost per chunk (4096 bytes at 1 byte/block), no compression of uniform regions.

### Palette-Based Compression

Used by modern Minecraft (post-1.13). Instead of storing raw block IDs, store indices into a local palette. The bit width of indices adapts to the number of unique block types in the chunk.

```rust
use bitvec::prelude::*;

pub struct PalettedChunk {
    /// Maps palette index -> actual BlockId
    palette: Vec<BlockId>,
    /// Packed bit array of palette indices, variable bits-per-entry
    data: BitVec<u64, Lsb0>,
    bits_per_entry: u8,
}

impl PalettedChunk {
    pub fn new_uniform(block: BlockId) -> Self {
        Self {
            palette: vec![block],
            data: bitvec![u64, Lsb0; 0; CHUNK_VOLUME],
            bits_per_entry: 1, // 1 bit per entry when only 1-2 types
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        let idx = Chunk::index(x, y, z);
        let start = idx * self.bits_per_entry as usize;
        let end = start + self.bits_per_entry as usize;
        let palette_idx = self.data[start..end].load::<usize>();
        self.palette[palette_idx]
    }

    /// When palette grows beyond threshold, expand bits_per_entry
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        let palette_idx = match self.palette.iter().position(|&b| b == block) {
            Some(idx) => idx,
            None => {
                self.palette.push(block);
                let new_idx = self.palette.len() - 1;
                // Check if we need to expand bit width
                if self.palette.len() > (1 << self.bits_per_entry) {
                    self.grow_bits();
                }
                new_idx
            }
        };
        let idx = Chunk::index(x, y, z);
        let start = idx * self.bits_per_entry as usize;
        let end = start + self.bits_per_entry as usize;
        self.data[start..end].store(palette_idx);
    }

    fn grow_bits(&mut self) {
        self.bits_per_entry += 1;
        // Rebuild data array with new bit width
        // ... (re-pack all existing indices with wider bit width)
    }
}
```

**Memory savings**: A chunk with only 2 block types (air + stone) uses 1 bit per block = 512 bytes vs 4096 bytes for flat array. Most underground chunks contain 3-5 types, using 2-3 bits = 1-1.5 KB.

**Single-type optimization**: Chunks containing only one block type (common for air-only chunks above terrain) can be stored as a single `BlockId` with zero data array allocation.

### Octree / Sparse Voxel Octree (SVO)

Tree-based structure where each node subdivides into 8 children. Best for highly sparse worlds or raytracing engines, but not ideal for Minecraft-like games.

```rust
pub enum OctreeNode {
    Leaf(BlockId),
    Branch(Box<[OctreeNode; 8]>),
    Empty,
}
```

**Pros**: Excellent for sparse data, natural LOD support, good for raytracing.
**Cons**: Pointer chasing kills cache performance, slow random access, complex meshing, not great for frequent edits.

### Recommendation

For a Minecraft-like game: **start with flat arrays, migrate to palette compression later**. Flat arrays are simplest to implement and mesh. Palette compression is a pure optimization that can be added without changing the public API.

---

## 2. Greedy Meshing Algorithm

Greedy meshing dramatically reduces triangle count by merging adjacent coplanar voxel faces into larger quads.

### How It Works

For each of the 6 face directions (±X, ±Y, ±Z), sweep a plane through the chunk:

1. **Build a 2D mask** of visible faces on the current slice
2. **Greedily merge** adjacent faces in the mask into maximal rectangles
3. **Emit one quad** per merged rectangle instead of one per voxel face

### Algorithm Pseudocode

```
for each axis (X, Y, Z):
    for each slice along that axis:
        // Build mask: which faces are visible on this slice?
        mask = 2D array of (block_type or NONE)
        for each (u, v) in slice:
            block_here = get_block(slice_pos)
            block_neighbor = get_block(slice_pos + axis_normal)
            if block_here is solid AND block_neighbor is not solid:
                mask[u][v] = block_here.type
            else:
                mask[u][v] = NONE

        // Greedy merge the mask into rectangles
        for v in 0..SIZE:
            u = 0
            while u < SIZE:
                if mask[u][v] == NONE:
                    u += 1
                    continue

                current_type = mask[u][v]

                // Find width: extend right while same type
                width = 1
                while u + width < SIZE AND mask[u + width][v] == current_type:
                    width += 1

                // Find height: extend down while entire row matches
                height = 1
                while v + height < SIZE:
                    row_matches = true
                    for i in 0..width:
                        if mask[u + i][v + height] != current_type:
                            row_matches = false
                            break
                    if not row_matches: break
                    height += 1

                // Emit quad for this rectangle
                emit_quad(position, width, height, axis, current_type)

                // Clear the merged region from the mask
                for dy in 0..height:
                    for dx in 0..width:
                        mask[u + dx][v + dy] = NONE

                u += width
```

### Rust Implementation Pattern

```rust
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub block_type: u32,
}

pub fn greedy_mesh(chunk: &Chunk) -> Vec<MeshVertex> {
    let mut vertices = Vec::new();
    let size = CHUNK_SIZE as i32;

    // For each axis direction
    for axis in 0..3 {
        let u_axis = (axis + 1) % 3;
        let v_axis = (axis + 2) % 3;

        // For both positive and negative face directions
        for &back_face in &[false, true] {
            // Sweep slices along this axis
            for slice in 0..size {
                // Build mask
                let mut mask = [[BlockId::AIR; CHUNK_SIZE]; CHUNK_SIZE];

                for v in 0..size {
                    for u in 0..size {
                        let mut pos = [0i32; 3];
                        pos[axis] = slice;
                        pos[u_axis] = u;
                        pos[v_axis] = v;

                        let block = chunk.get(pos[0] as usize, pos[1] as usize, pos[2] as usize);

                        // Check neighbor in axis direction
                        let mut neighbor_pos = pos;
                        neighbor_pos[axis] += if back_face { -1 } else { 1 };

                        let neighbor = if neighbor_pos[axis] < 0 || neighbor_pos[axis] >= size {
                            BlockId::AIR // chunk boundary - need neighbor chunk data
                        } else {
                            chunk.get(
                                neighbor_pos[0] as usize,
                                neighbor_pos[1] as usize,
                                neighbor_pos[2] as usize,
                            )
                        };

                        if block.is_solid() && !neighbor.is_solid() {
                            mask[v as usize][u as usize] = block;
                        }
                    }
                }

                // Greedy merge mask into quads
                for v in 0..CHUNK_SIZE {
                    let mut u = 0;
                    while u < CHUNK_SIZE {
                        let block = mask[v][u];
                        if block == BlockId::AIR {
                            u += 1;
                            continue;
                        }

                        // Extend width
                        let mut w = 1;
                        while u + w < CHUNK_SIZE && mask[v][u + w] == block {
                            w += 1;
                        }

                        // Extend height
                        let mut h = 1;
                        'outer: while v + h < CHUNK_SIZE {
                            for i in 0..w {
                                if mask[v + h][u + i] != block {
                                    break 'outer;
                                }
                            }
                            h += 1;
                        }

                        // Emit quad vertices
                        // ... (build 4 vertices for the w x h rectangle)

                        // Clear merged region
                        for dy in 0..h {
                            for dx in 0..w {
                                mask[v + dy][u + dx] = BlockId::AIR;
                            }
                        }

                        u += w;
                    }
                }
            }
        }
    }

    vertices
}
```

### Binary Greedy Meshing (Advanced)

An optimized variant that uses 64-bit integers and bitwise operations to process 64 voxel faces simultaneously. The `binary-greedy-meshing` crate on crates.io implements this and is ~30x faster than the standard `block-mesh-rs` approach.

Key insight: represent each row of the mask as a `u64` bitmask, then use bit manipulation to find runs of set bits and merge them.

### Performance Comparison

| Method | Quads for typical chunk | Time |
|--------|------------------------|------|
| Naive (1 quad per face) | ~8000-12000 | <1ms |
| Greedy meshing | ~800-2000 | ~1-3ms |
| Binary greedy meshing | ~800-2000 | ~0.05-0.1ms |

---

## 3. Face Culling

Only generate mesh faces at boundaries between solid and non-solid blocks. This is the single most impactful optimization.

### Internal Face Culling

Within a chunk, check each block against its 6 neighbors:

```rust
pub fn should_generate_face(chunk: &Chunk, x: i32, y: i32, z: i32, dir: Direction) -> bool {
    let (nx, ny, nz) = dir.offset();
    let (check_x, check_y, check_z) = (x + nx, y + ny, z + nz);

    // If neighbor is outside chunk bounds, check neighbor chunk
    if !in_bounds(check_x, check_y, check_z) {
        return true; // Generate face at chunk boundary (or check neighbor chunk)
    }

    let neighbor = chunk.get(check_x as usize, check_y as usize, check_z as usize);
    !neighbor.is_solid() || neighbor.is_transparent()
}
```

### Cross-Chunk Face Culling

Faces at chunk boundaries require access to the neighboring chunk's data. Two approaches:

1. **Padding**: Store a 1-block border from each neighbor chunk (18x18x18 for a 16x16x16 chunk). This is what `bevy_voxel_world` does (34x34x34 for 32x32x32 chunks). Avoids locking neighbor chunks during meshing.

2. **Lazy boundary**: Generate boundary faces optimistically, re-mesh when neighbor chunk loads. Simpler but causes visual pop-in.

### Transparent Block Handling

Transparent blocks (glass, water, leaves) need special rules:
- Generate faces between solid and transparent blocks
- Generate faces between transparent and air blocks
- Do NOT generate faces between two transparent blocks of the same type (avoids internal water/glass faces)

```rust
pub fn should_generate_face_between(block: BlockId, neighbor: BlockId) -> bool {
    if !block.is_solid() && !block.is_transparent() {
        return false; // Air generates no faces
    }
    if neighbor.is_air() {
        return true; // Always show face toward air
    }
    if block.is_solid() && neighbor.is_transparent() {
        return true; // Solid face visible through transparent neighbor
    }
    if block.is_transparent() && block != neighbor {
        return true; // Different transparent types show boundary
    }
    false
}
```

---

## 4. Chunk Loading/Unloading Strategy

### Distance-Based Spawning with Priority Queue

```rust
use std::collections::{HashMap, BinaryHeap};
use bevy::prelude::*;

pub const RENDER_DISTANCE: i32 = 12; // chunks
pub const DESPAWN_DISTANCE: i32 = 14; // slightly larger to avoid thrashing
pub const MAX_CHUNKS_PER_FRAME: usize = 4; // limit work per frame

#[derive(Resource)]
pub struct ChunkManager {
    /// All currently loaded chunks
    loaded: HashMap<IVec3, Entity>,
    /// Priority queue of chunks to load (closest first)
    load_queue: BinaryHeap<ChunkLoadRequest>,
}

#[derive(Eq, PartialEq)]
struct ChunkLoadRequest {
    position: IVec3,
    priority: i32, // negative distance (BinaryHeap is max-heap)
}

impl Ord for ChunkLoadRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority) // higher priority = closer
    }
}

impl PartialOrd for ChunkLoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn update_chunk_loading(
    mut chunk_manager: ResMut<ChunkManager>,
    camera: Query<&Transform, With<Camera>>,
    mut commands: Commands,
) {
    let cam_pos = camera.single().translation;
    let cam_chunk = world_to_chunk_pos(cam_pos);

    // Queue new chunks within render distance
    for x in -RENDER_DISTANCE..=RENDER_DISTANCE {
        for y in -RENDER_DISTANCE..=RENDER_DISTANCE {
            for z in -RENDER_DISTANCE..=RENDER_DISTANCE {
                let chunk_pos = cam_chunk + IVec3::new(x, y, z);
                let dist = (chunk_pos - cam_chunk).as_vec3().length() as i32;
                if dist <= RENDER_DISTANCE && !chunk_manager.loaded.contains_key(&chunk_pos) {
                    chunk_manager.load_queue.push(ChunkLoadRequest {
                        position: chunk_pos,
                        priority: -dist, // negate for min-heap behavior
                    });
                }
            }
        }
    }

    // Process load queue (limited per frame)
    let mut loaded_this_frame = 0;
    while let Some(request) = chunk_manager.load_queue.pop() {
        if loaded_this_frame >= MAX_CHUNKS_PER_FRAME {
            break;
        }
        if chunk_manager.loaded.contains_key(&request.position) {
            continue; // Already loaded
        }

        // Spawn chunk entity and kick off async generation
        let entity = commands.spawn(ChunkBundle::new(request.position)).id();
        chunk_manager.loaded.insert(request.position, entity);
        loaded_this_frame += 1;
    }

    // Despawn distant chunks
    let to_despawn: Vec<IVec3> = chunk_manager.loaded.keys()
        .filter(|pos| {
            let dist = (**pos - cam_chunk).as_vec3().length() as i32;
            dist > DESPAWN_DISTANCE
        })
        .cloned()
        .collect();

    for pos in to_despawn {
        if let Some(entity) = chunk_manager.loaded.remove(&pos) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### Hysteresis (Despawn > Render Distance)

The despawn distance must be greater than the render distance to prevent "thrashing" - rapidly spawning and despawning chunks when the player stands near a boundary. A gap of 2 chunks is typical.

### Spiral Loading Pattern

Instead of iterating a cubic volume, load chunks in a spiral outward from the camera. This ensures the closest chunks load first without needing a priority queue:

```rust
pub fn spiral_chunk_positions(radius: i32) -> Vec<IVec3> {
    let mut positions: Vec<IVec3> = Vec::new();
    for x in -radius..=radius {
        for y in -radius..=radius {
            for z in -radius..=radius {
                let pos = IVec3::new(x, y, z);
                if pos.as_vec3().length() <= radius as f32 {
                    positions.push(pos);
                }
            }
        }
    }
    // Sort by distance from origin (closest first)
    positions.sort_by(|a, b| {
        a.as_vec3().length_squared()
            .partial_cmp(&b.as_vec3().length_squared())
            .unwrap()
    });
    positions
}
```

---

## 5. Multi-Threaded Chunk Generation

### Bevy AsyncComputeTaskPool Pattern

Bevy provides `AsyncComputeTaskPool` for CPU-intensive background work that doesn't need to complete within the current frame. This is ideal for chunk meshing.

```rust
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

/// Marker component for chunks that need meshing
#[derive(Component)]
pub struct NeedsMesh;

/// Component holding the async meshing task
#[derive(Component)]
pub struct MeshTask(Task<Mesh>);

/// System: kick off async mesh generation for chunks that need it
pub fn start_mesh_tasks(
    mut commands: Commands,
    chunks: Query<(Entity, &ChunkData), With<NeedsMesh>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, chunk_data) in &chunks {
        let data = chunk_data.clone(); // Clone data for the async task

        let task = task_pool.spawn(async move {
            // This runs on a background thread
            greedy_mesh(&data)
        });

        commands.entity(entity)
            .remove::<NeedsMesh>()
            .insert(MeshTask(task));
    }
}

/// System: poll completed mesh tasks and apply results
pub fn apply_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tasks: Query<(Entity, &mut MeshTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mesh) = future::block_on(future::poll_once(&mut task.0)) {
            let mesh_handle = meshes.add(mesh);
            commands.entity(entity)
                .remove::<MeshTask>()
                .insert(Mesh3d(mesh_handle));
        }
    }
}
```

### System Ordering

```rust
app.add_systems(Update, (
    update_chunk_loading,
    start_mesh_tasks.after(update_chunk_loading),
    apply_mesh_results.after(start_mesh_tasks),
));
```

### Parallel Terrain Generation with Rayon (Alternative)

For initial world generation where many chunks are computed at once:

```rust
use rayon::prelude::*;

pub fn generate_chunks_parallel(positions: &[IVec3]) -> Vec<(IVec3, Chunk)> {
    positions.par_iter()
        .map(|&pos| {
            let chunk = generate_terrain(pos);
            (pos, chunk)
        })
        .collect()
}
```

### Thread Safety Considerations

- Chunk data must be `Send + Sync` for async tasks
- Use `Arc<RwLock<ChunkMap>>` for shared chunk access
- Use write buffers to batch modifications (like `bevy_voxel_world`'s `VoxelWriteBuffer`)
- Avoid locking during meshing - clone/snapshot chunk data before sending to worker

---

## 6. Collision Detection with Voxels

### AABB vs Voxel Grid

The standard approach for Minecraft-like games: represent entities as axis-aligned bounding boxes (AABBs) and check them against the voxel grid.

```rust
#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    pub fn overlaps_block(&self, block_pos: IVec3) -> bool {
        let block_min = block_pos.as_vec3();
        let block_max = block_min + Vec3::ONE;

        self.min.x < block_max.x && self.max.x > block_min.x &&
        self.min.y < block_max.y && self.max.y > block_min.y &&
        self.min.z < block_max.z && self.max.z > block_min.z
    }
}
```

### Swept AABB Collision

For correct collision response, use swept collision that finds the exact point of contact along the movement vector:

```rust
pub struct CollisionResult {
    /// How far along the velocity we can move (0.0 to 1.0)
    pub time: f32,
    /// Which face we hit
    pub normal: Vec3,
}

/// Sweep an AABB along a velocity vector, checking against solid voxels
pub fn sweep_aabb(
    aabb: &Aabb,
    velocity: Vec3,
    world: &VoxelWorld,
) -> Option<CollisionResult> {
    // 1. Compute the broadphase AABB (union of start and end positions)
    let swept_min = aabb.min.min(aabb.min + velocity);
    let swept_max = aabb.max.max(aabb.max + velocity);

    // 2. Find all solid blocks within the broadphase
    let block_min = swept_min.floor().as_ivec3();
    let block_max = swept_max.ceil().as_ivec3();

    let mut earliest_hit: Option<CollisionResult> = None;

    for x in block_min.x..=block_max.x {
        for y in block_min.y..=block_max.y {
            for z in block_min.z..=block_max.z {
                let block_pos = IVec3::new(x, y, z);
                if !world.is_solid(block_pos) {
                    continue;
                }

                // 3. Swept AABB vs block AABB test
                let block_aabb = Aabb::new(
                    block_pos.as_vec3() + Vec3::splat(0.5),
                    Vec3::splat(0.5),
                );

                if let Some(hit) = swept_aabb_vs_aabb(aabb, velocity, &block_aabb) {
                    match &earliest_hit {
                        None => earliest_hit = Some(hit),
                        Some(prev) if hit.time < prev.time => earliest_hit = Some(hit),
                        _ => {}
                    }
                }
            }
        }
    }

    earliest_hit
}

/// Resolve collisions axis-by-axis for stable "sliding" behavior
pub fn move_and_slide(
    aabb: &mut Aabb,
    velocity: &mut Vec3,
    world: &VoxelWorld,
) {
    // Resolve each axis independently to allow sliding along walls
    for axis in 0..3 {
        let mut axis_vel = Vec3::ZERO;
        axis_vel[axis] = velocity[axis];

        if let Some(hit) = sweep_aabb(aabb, axis_vel, world) {
            // Move to contact point
            let move_dist = axis_vel * hit.time;
            aabb.min += move_dist;
            aabb.max += move_dist;
            velocity[axis] = 0.0; // Stop movement on this axis
        } else {
            // No collision, apply full movement on this axis
            aabb.min += axis_vel;
            aabb.max += axis_vel;
        }
    }
}
```

### Key Design Decisions

- **Axis-by-axis resolution**: Resolve X, Y, Z independently to enable sliding along walls/floors
- **Broadphase**: Only check blocks in the swept AABB region, not the entire world
- **Step-up**: Minecraft allows walking up 0.5-block steps automatically. Implement as: if horizontal collision, try moving up by step height and re-check
- **Gravity**: Apply as a constant downward velocity each frame, resolved by the Y-axis collision

---

## 7. Block Updates and Mesh Rebuilds

When a block changes, the chunk containing it (and potentially neighboring chunks) need re-meshing.

### Write Buffer Pattern (from bevy_voxel_world)

```rust
/// Buffer for block changes that haven't been applied yet
#[derive(Resource, Default)]
pub struct BlockWriteBuffer {
    writes: Vec<(IVec3, BlockId)>,
}

impl BlockWriteBuffer {
    pub fn set_block(&mut self, world_pos: IVec3, block: BlockId) {
        self.writes.push((world_pos, block));
    }
}

/// System: flush write buffer and mark affected chunks for re-meshing
pub fn flush_block_writes(
    mut buffer: ResMut<BlockWriteBuffer>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands,
) {
    for (world_pos, block) in buffer.writes.drain(..) {
        let chunk_pos = world_to_chunk_pos(world_pos.as_vec3());
        let local_pos = world_to_local_pos(world_pos);

        // Apply the block change
        if let Some(chunk) = chunk_manager.get_chunk_mut(chunk_pos) {
            chunk.set(local_pos.x as usize, local_pos.y as usize, local_pos.z as usize, block);
        }

        // Mark this chunk for re-mesh
        if let Some(&entity) = chunk_manager.loaded.get(&chunk_pos) {
            commands.entity(entity).insert(NeedsMesh);
        }

        // Check if block is on a chunk boundary - also re-mesh neighbors
        let local = local_pos;
        if local.x == 0 {
            mark_needs_mesh(&chunk_manager, &mut commands, chunk_pos + IVec3::NEG_X);
        }
        if local.x == CHUNK_SIZE as i32 - 1 {
            mark_needs_mesh(&chunk_manager, &mut commands, chunk_pos + IVec3::X);
        }
        if local.y == 0 {
            mark_needs_mesh(&chunk_manager, &mut commands, chunk_pos + IVec3::NEG_Y);
        }
        if local.y == CHUNK_SIZE as i32 - 1 {
            mark_needs_mesh(&chunk_manager, &mut commands, chunk_pos + IVec3::Y);
        }
        if local.z == 0 {
            mark_needs_mesh(&chunk_manager, &mut commands, chunk_pos + IVec3::NEG_Z);
        }
        if local.z == CHUNK_SIZE as i32 - 1 {
            mark_needs_mesh(&chunk_manager, &mut commands, chunk_pos + IVec3::Z);
        }
    }
}

fn mark_needs_mesh(
    chunk_manager: &ChunkManager,
    commands: &mut Commands,
    chunk_pos: IVec3,
) {
    if let Some(&entity) = chunk_manager.loaded.get(&chunk_pos) {
        commands.entity(entity).insert(NeedsMesh);
    }
}
```

### Debouncing

If many blocks change in rapid succession (explosion, world edit), debounce re-meshing to avoid redundant work. Only re-mesh once per frame per chunk.

---

## 8. Frustum Culling

Bevy has built-in frustum culling for entities with `Aabb` components. For a voxel engine, each chunk entity gets an AABB and Bevy automatically skips rendering chunks outside the camera's view frustum.

### Built-in Bevy Approach

```rust
use bevy::render::primitives::Aabb;

/// When spawning a chunk entity, add an AABB for frustum culling
fn spawn_chunk(commands: &mut Commands, chunk_pos: IVec3) -> Entity {
    let world_pos = chunk_pos.as_vec3() * CHUNK_SIZE as f32;

    commands.spawn((
        Transform::from_translation(world_pos),
        Aabb::from_min_max(
            Vec3::ZERO,
            Vec3::splat(CHUNK_SIZE as f32),
        ),
        // ... mesh, material, etc.
    )).id()
}
```

Bevy's rendering pipeline automatically performs frustum culling using these AABBs, so no custom culling code is needed for basic visibility. However, you may want additional culling to skip meshing for off-screen chunks entirely.

### Mesh Generation Culling

Only queue mesh generation for chunks that are within or near the frustum:

```rust
pub fn should_mesh_chunk(chunk_pos: IVec3, camera_transform: &Transform, fov: f32) -> bool {
    let chunk_center = chunk_pos.as_vec3() * CHUNK_SIZE as f32 + Vec3::splat(CHUNK_SIZE as f32 / 2.0);
    let to_chunk = chunk_center - camera_transform.translation;
    let forward = camera_transform.forward();

    // Dot product check: is chunk roughly in front of camera?
    let dot = to_chunk.normalize().dot(*forward);
    dot > -0.2 // Allow some margin behind camera for chunks partially in view
}
```

---

## 9. Memory Management for Infinite Worlds

### Two-Layer Architecture (from bevy_voxel_world)

1. **Procedural layer**: Terrain generation function produces voxels on-demand. Never stored permanently. Re-generated when chunk loads.
2. **Modification layer**: A `HashMap<IVec3, BlockId>` stores only player-modified blocks. Persisted to disk.

This means the vast majority of the world (unmodified terrain) costs zero memory when unloaded.

```rust
#[derive(Resource, Default)]
pub struct ModifiedBlocks {
    /// Only stores blocks that differ from procedurally generated terrain
    modifications: HashMap<IVec3, BlockId>,
}

pub fn get_block(world_pos: IVec3, modifications: &ModifiedBlocks) -> BlockId {
    // Check modifications first
    if let Some(&block) = modifications.modifications.get(&world_pos) {
        return block;
    }
    // Fall back to procedural generation
    generate_block_at(world_pos)
}
```

### Chunk Unloading Strategy

- Keep a fixed budget of loaded chunks (e.g., ~10,000)
- When budget is exceeded, unload furthest chunks first
- Save modified chunks before unloading
- Use LRU cache semantics: recently accessed chunks stay loaded

### Memory Budget Estimation

For 16x16x16 chunks at 1 byte/block:
- Per chunk: 4,096 bytes data + ~200 bytes metadata = ~4.3 KB
- Render distance 12: ~7,000 chunks within sphere = ~30 MB chunk data
- Mesh data: ~2-10 KB per chunk mesh = ~14-70 MB
- Total: ~50-100 MB for chunk data + meshes (very manageable)

---

## 10. Save/Load Format

### Custom Binary Format (Recommended over Anvil)

Minecraft's Anvil format is complex and designed for Java's NBT serialization. For a Rust engine, a simpler custom format works better.

### Region-Based File System

Inspired by Minecraft's region files but simplified:

```rust
use serde::{Serialize, Deserialize};

/// A region contains 16x16x16 chunks
pub const REGION_SIZE: i32 = 16;

#[derive(Serialize, Deserialize)]
pub struct RegionFile {
    /// Which chunks in this region have been modified
    /// Key: local chunk position within region (0..16 on each axis)
    chunks: HashMap<IVec3, ChunkSaveData>,
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSaveData {
    /// Only stores modifications - blocks that differ from procedural generation
    modified_blocks: Vec<(LocalBlockPos, BlockId)>,
    /// Entity data within this chunk (items, mobs, etc.)
    entities: Vec<EntitySaveData>,
}

/// File naming: region_X_Y_Z.bin
pub fn region_file_path(region_pos: IVec3) -> String {
    format!("world/regions/r.{}.{}.{}.bin", region_pos.x, region_pos.y, region_pos.z)
}

pub fn chunk_to_region_pos(chunk_pos: IVec3) -> IVec3 {
    IVec3::new(
        chunk_pos.x.div_euclid(REGION_SIZE),
        chunk_pos.y.div_euclid(REGION_SIZE),
        chunk_pos.z.div_euclid(REGION_SIZE),
    )
}
```

### Serialization with Compression

```rust
use std::io::{Read, Write};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;

pub fn save_region(region: &RegionFile, path: &str) -> std::io::Result<()> {
    let encoded = bincode::serialize(region).unwrap();
    let file = std::fs::File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::fast());
    encoder.write_all(&encoded)?;
    encoder.finish()?;
    Ok(())
}

pub fn load_region(path: &str) -> std::io::Result<RegionFile> {
    let file = std::fs::File::open(path)?;
    let mut decoder = GzDecoder::new(file);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(bincode::deserialize(&bytes).unwrap())
}
```

### Auto-Save Strategy

- Save modified regions periodically (every 30-60 seconds)
- Save when chunks are unloaded from memory
- Save all on game exit
- Track dirty flags per region to avoid unnecessary I/O

---

## 11. Coordinate Systems

### Three Coordinate Spaces

```rust
/// World coordinates: absolute position of a block in the world
/// Range: -2^31 to 2^31 (practically infinite)
pub type WorldPos = IVec3;

/// Chunk coordinates: which chunk a block belongs to
/// chunk_pos = world_pos / CHUNK_SIZE (using integer floor division)
pub type ChunkPos = IVec3;

/// Local coordinates: position of a block within its chunk
/// Range: 0..CHUNK_SIZE on each axis
pub type LocalPos = UVec3;

/// Convert world position to chunk position
/// IMPORTANT: Use div_euclid, not regular division, for correct negative handling
pub fn world_to_chunk_pos(world_pos: Vec3) -> IVec3 {
    let size = CHUNK_SIZE as i32;
    IVec3::new(
        (world_pos.x as i32).div_euclid(size),
        (world_pos.y as i32).div_euclid(size),
        (world_pos.z as i32).div_euclid(size),
    )
}

/// Convert world position to local block position within chunk
pub fn world_to_local_pos(world_pos: IVec3) -> UVec3 {
    let size = CHUNK_SIZE as i32;
    UVec3::new(
        world_pos.x.rem_euclid(size) as u32,
        world_pos.y.rem_euclid(size) as u32,
        world_pos.z.rem_euclid(size) as u32,
    )
}

/// Convert chunk position + local position back to world position
pub fn chunk_local_to_world(chunk_pos: IVec3, local_pos: UVec3) -> IVec3 {
    chunk_pos * CHUNK_SIZE as i32 + local_pos.as_ivec3()
}
```

### Critical: `div_euclid` vs Regular Division

Regular integer division rounds toward zero, which gives wrong results for negative coordinates:

```
// Regular division: -7 / 16 = 0  (WRONG: block at -7 is in chunk -1)
// div_euclid:       -7 / 16 = -1 (CORRECT)

// Regular modulo:   -7 % 16 = -7  (WRONG: local position can't be negative)
// rem_euclid:       -7 % 16 = 9   (CORRECT)
```

This is one of the most common bugs in voxel engine implementations. Always use `div_euclid` and `rem_euclid`.

### Floating Point World Position

For entity positions and rendering:

```rust
/// Convert a floating-point world position to the block it's in
pub fn float_to_block_pos(pos: Vec3) -> IVec3 {
    IVec3::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    )
}
```

---

## 12. Existing Rust Voxel Engines

### vx_bevy (Game4all/vx_bevy)
- Minecraft-esque prototype built on Bevy
- Uses greedy meshing for chunk rendering (one triangle mesh per chunk)
- Good learning reference but not actively maintained on latest Bevy

### building-blocks (bonsairobo/building-blocks)
- Comprehensive voxel library for real-time applications
- Provides lattice map data structures, mesh generation, chunk compression (Lz4, Snappy)
- Now in maintenance mode; author extracted key features into standalone crates:
  - `block-mesh-rs`: Fast meshing algorithms (greedy quads, simple quads, height maps)
  - These are the recommended crates to use

### block-mesh-rs (bonsairobo/block-mesh-rs)
- The meshing component extracted from building-blocks
- Provides `greedy_quads()` and `visible_block_faces()` algorithms
- Well-tested and optimized, good production choice
- Generates ~1/3 the quads of naive meshing but takes ~3x longer than naive

### binary-greedy-meshing (crates.io)
- Port of the Binary Greedy Meshing v2 algorithm to Rust
- ~30x faster than block-mesh-rs for greedy meshing
- Uses bitwise operations on u64 masks to process 64 faces simultaneously
- Supports transparent blocks

### bevy_voxel_world (splashdust/bevy_voxel_world)
- Full-featured Bevy plugin for voxel worlds
- Handles: async meshing, chunk spawn/despawn, texture mapping, LOD
- Two-layer architecture: procedural base + modification overlay
- Uses `Arc<RwLock>` for thread-safe chunk access
- Event-driven lifecycle: ChunkWillSpawn, ChunkWillRemesh, ChunkWillDespawn
- Best reference for Bevy integration patterns

### Veloren (veloren/veloren)
- Large-scale multiplayer voxel RPG
- Uses specs ECS (not Bevy), wgpu rendering
- Implements greedy meshing with special handling for chunk boundary faces
- Voxels represented with minimal data; very game-specific optimizations
- Client-server architecture with ECS on both sides

### Meshem (Adamkob12/Meshem)
- Bevy-specific meshing crate
- Provides multiple meshing strategies for voxel grids
- Designed to integrate directly into Bevy's ECS

### projekto (afonsolage/projekto)
- Voxel game built on Bevy
- Another reference for Bevy + voxel integration

---

## Summary: Recommended Architecture Stack

For a Minecraft-like game built with Bevy:

| Component | Recommendation |
|-----------|---------------|
| Chunk data | Start with flat `[BlockId; 4096]` arrays, upgrade to palette later |
| Chunk size | 16x16x16 (Minecraft standard, good balance of mesh size vs overhead) |
| Meshing | `block-mesh-rs` for greedy quads, or `binary-greedy-meshing` if speed matters |
| Threading | Bevy `AsyncComputeTaskPool` for chunk gen + meshing |
| Chunk loading | Distance-based with priority queue, hysteresis on despawn |
| Collision | Swept AABB, axis-by-axis resolution |
| Coordinates | `div_euclid`/`rem_euclid` for all world-to-chunk conversions |
| Storage | Two-layer (procedural + modifications), region files with bincode + gzip |
| Culling | Bevy built-in frustum culling via AABB components + face culling during meshing |
| World plugin | Consider `bevy_voxel_world` as starting point, or build custom for more control |
