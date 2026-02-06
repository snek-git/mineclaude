use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

use crate::block::atlas::{face_uvs_tiled, tile_uvs, texture_index};
use crate::block::{BlockType, Face};
use crate::world::chunk::{Chunk, CHUNK_SIZE};

/// Optional neighbor chunk data for cross-chunk face culling.
/// Order: [+X, -X, +Y, -Y, +Z, -Z] matching [East, West, Top, Bottom, South, North].
pub type NeighborChunks<'a> = [Option<&'a Chunk>; 6];

/// Describes a face direction for the sweeping algorithm.
struct FaceDir {
    /// Which Face enum variant this corresponds to
    face: Face,
    /// The axis we sweep along (0=X, 1=Y, 2=Z)
    axis: usize,
    /// The first tangent axis (u-axis of the 2D mask)
    u_axis: usize,
    /// The second tangent axis (v-axis of the 2D mask)
    v_axis: usize,
    /// Whether the face points in the negative direction along the axis
    back_face: bool,
    /// Whether to flip triangle winding to get correct CCW order
    flip_winding: bool,
}

const FACE_DIRS: [FaceDir; 6] = [
    FaceDir { face: Face::East,   axis: 0, u_axis: 2, v_axis: 1, back_face: false, flip_winding: true },
    FaceDir { face: Face::West,   axis: 0, u_axis: 2, v_axis: 1, back_face: true,  flip_winding: true },
    FaceDir { face: Face::Top,    axis: 1, u_axis: 0, v_axis: 2, back_face: false, flip_winding: true },
    FaceDir { face: Face::Bottom, axis: 1, u_axis: 0, v_axis: 2, back_face: true,  flip_winding: true },
    FaceDir { face: Face::South,  axis: 2, u_axis: 0, v_axis: 1, back_face: false, flip_winding: false },
    FaceDir { face: Face::North,  axis: 2, u_axis: 0, v_axis: 1, back_face: true,  flip_winding: false },
];

/// Determines whether a face should be generated between two blocks.
/// `block` is the block that owns the face, `neighbor` is on the other side.
#[inline]
fn should_emit_face(block: BlockType, neighbor: BlockType) -> bool {
    // Air blocks never generate faces
    if block == BlockType::Air {
        return false;
    }
    // Non-cube blocks (torch, tall grass, saplings) are not rendered as cubes
    if block.is_non_cube() {
        return false;
    }
    // Non-solid, non-transparent blocks generate nothing (only Air matches this)
    if !block.is_solid() && !block.is_transparent() {
        return false;
    }
    // Face toward air is always visible
    if neighbor == BlockType::Air {
        return true;
    }
    // Solid opaque face visible through a transparent neighbor
    if block.is_solid() && !block.is_transparent() && neighbor.is_transparent() {
        return true;
    }
    // Transparent block: show face when neighbor is a different type
    if block.is_transparent() && block != neighbor {
        return true;
    }
    false
}

/// Gets the block at position `[axis, u, v]` remapped to `[x, y, z]`.
#[inline]
fn get_block_from_pos(chunk: &Chunk, axis: usize, u_axis: usize, v_axis: usize, a: usize, u: usize, v: usize) -> BlockType {
    let mut pos = [0usize; 3];
    pos[axis] = a;
    pos[u_axis] = u;
    pos[v_axis] = v;
    chunk.get(pos[0], pos[1], pos[2])
}

/// Gets the neighbor block for a face. If the neighbor is outside the chunk,
/// uses the neighbor chunk data if available, otherwise returns Air.
#[inline]
fn get_neighbor_block(
    chunk: &Chunk,
    neighbors: &NeighborChunks,
    face_dir: &FaceDir,
    slice: usize,
    u: usize,
    v: usize,
) -> BlockType {
    let size = CHUNK_SIZE;
    if face_dir.back_face {
        // Neighbor is at slice - 1 along the axis
        if slice == 0 {
            // Need neighbor chunk in negative direction
            let neighbor_idx = match face_dir.face {
                Face::West => 1,   // -X
                Face::Bottom => 3, // -Y
                Face::North => 5,  // -Z
                _ => unreachable!(),
            };
            if let Some(neighbor_chunk) = neighbors[neighbor_idx] {
                let mut pos = [0usize; 3];
                pos[face_dir.axis] = size - 1;
                pos[face_dir.u_axis] = u;
                pos[face_dir.v_axis] = v;
                neighbor_chunk.get(pos[0], pos[1], pos[2])
            } else {
                BlockType::Air
            }
        } else {
            get_block_from_pos(chunk, face_dir.axis, face_dir.u_axis, face_dir.v_axis, slice - 1, u, v)
        }
    } else {
        // Neighbor is at slice + 1 along the axis
        if slice == size - 1 {
            let neighbor_idx = match face_dir.face {
                Face::East => 0,  // +X
                Face::Top => 2,   // +Y
                Face::South => 4, // +Z
                _ => unreachable!(),
            };
            if let Some(neighbor_chunk) = neighbors[neighbor_idx] {
                let mut pos = [0usize; 3];
                pos[face_dir.axis] = 0;
                pos[face_dir.u_axis] = u;
                pos[face_dir.v_axis] = v;
                neighbor_chunk.get(pos[0], pos[1], pos[2])
            } else {
                BlockType::Air
            }
        } else {
            get_block_from_pos(chunk, face_dir.axis, face_dir.u_axis, face_dir.v_axis, slice + 1, u, v)
        }
    }
}

/// Build a chunk mesh using greedy meshing with face culling.
///
/// Takes a reference to the chunk and optional neighbor chunks for cross-chunk
/// face culling. Returns a Bevy `Mesh` with positions, normals, UVs, and indices.
pub fn build_chunk_mesh(chunk: &Chunk, neighbors: &NeighborChunks) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut uv1s: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    if chunk.is_empty() {
        return empty_mesh();
    }

    let size = CHUNK_SIZE;

    for face_dir in &FACE_DIRS {
        let normal = face_dir.face.normal();

        // Sweep slices along the main axis
        for slice in 0..size {
            // Build 2D mask: which block type (or None) has a visible face here
            let mut mask = [[BlockType::Air; CHUNK_SIZE]; CHUNK_SIZE];

            for v in 0..size {
                for u in 0..size {
                    let block = get_block_from_pos(chunk, face_dir.axis, face_dir.u_axis, face_dir.v_axis, slice, u, v);
                    let neighbor = get_neighbor_block(chunk, neighbors, face_dir, slice, u, v);

                    if should_emit_face(block, neighbor) {
                        mask[v][u] = block;
                    }
                }
            }

            // Greedy merge the mask into rectangles
            for v in 0..size {
                let mut u = 0;
                while u < size {
                    let block = mask[v][u];
                    if block == BlockType::Air {
                        u += 1;
                        continue;
                    }

                    // Transparent blocks (glass, water, leaves) must not be merged —
                    // merging removes internal faces between adjacent transparent blocks.
                    let (w, h) = if block.is_transparent() {
                        mask[v][u] = BlockType::Air;
                        (1, 1)
                    } else {
                        // Greedy merge: expand width along u-axis
                        let mut w = 1;
                        while u + w < size && mask[v][u + w] == block {
                            w += 1;
                        }

                        // Expand height along v-axis (all cells in the row must match)
                        let mut h = 1;
                        'outer: while v + h < size {
                            for du in 0..w {
                                if mask[v + h][u + du] != block {
                                    break 'outer;
                                }
                            }
                            h += 1;
                        }

                        // Clear the merged region from the mask
                        for dv in 0..h {
                            for du in 0..w {
                                mask[v + dv][u + du] = BlockType::Air;
                            }
                        }
                        (w, h)
                    };

                    // Emit quad for the merged rectangle
                    emit_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut uv1s,
                        &mut indices,
                        face_dir,
                        slice,
                        u,
                        v,
                        w,
                        h,
                        normal,
                        block,
                    );

                    u += w;
                }
            }
        }
    }

    // Second pass: cross-billboard meshes for non-cube blocks (torch, tallgrass, saplings, wheat)
    for y in 0..size {
        for z in 0..size {
            for x in 0..size {
                let block = chunk.get(x, y, z);
                if !block.is_non_cube() {
                    continue;
                }

                emit_cross_billboard(
                    &mut positions,
                    &mut normals,
                    &mut uvs,
                    &mut uv1s,
                    &mut indices,
                    x as f32,
                    y as f32,
                    z as f32,
                    block,
                );
            }
        }
    }

    if positions.is_empty() {
        return empty_mesh();
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, uv1s)
        .with_inserted_indices(Indices::U32(indices))
}

/// Emit 4 vertices and 6 indices for a greedy-merged quad.
#[allow(clippy::too_many_arguments)]
fn emit_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    uv1s: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    face_dir: &FaceDir,
    slice: usize,
    u_start: usize,
    v_start: usize,
    width: usize,
    height: usize,
    normal: [f32; 3],
    block: BlockType,
) {
    let base_index = positions.len() as u32;

    // Compute the 4 corner positions of the quad.
    // The quad lies on the face plane, which is at:
    //   slice (for back faces) or slice+1 (for front faces) along the axis.
    let face_offset = if face_dir.back_face { slice as f32 } else { (slice + 1) as f32 };

    // The 4 corners in (u, v) space: (u_start, v_start) to (u_start+width, v_start+height)
    let u0 = u_start as f32;
    let v0 = v_start as f32;
    let u1 = (u_start + width) as f32;
    let v1 = (v_start + height) as f32;

    // Map (axis=face_offset, u, v) back to (x, y, z)
    let corner = |u: f32, v: f32| -> [f32; 3] {
        let mut pos = [0.0f32; 3];
        pos[face_dir.axis] = face_offset;
        pos[face_dir.u_axis] = u;
        pos[face_dir.v_axis] = v;
        pos
    };

    // Determine the 4 vertices in correct winding order.
    // For front faces (positive normal), counter-clockwise when looking at the face.
    // For back faces (negative normal), reverse winding.
    let (v0_pos, v1_pos, v2_pos, v3_pos) = if face_dir.back_face {
        // Back face: flip winding
        (
            corner(u0, v0),
            corner(u0, v1),
            corner(u1, v1),
            corner(u1, v0),
        )
    } else {
        // Front face: standard CCW
        (
            corner(u0, v0),
            corner(u1, v0),
            corner(u1, v1),
            corner(u0, v1),
        )
    };

    positions.push(v0_pos);
    positions.push(v1_pos);
    positions.push(v2_pos);
    positions.push(v3_pos);

    normals.push(normal);
    normals.push(normal);
    normals.push(normal);
    normals.push(normal);

    // UV mapping: tile the texture across the merged greedy quad.
    // UVs span width * tile_size along u and height * tile_size along v,
    // so the texture repeats naturally across the merged region.
    let [uv_bl, uv_br, uv_tr, uv_tl] = face_uvs_tiled(block, face_dir.face, width, height);

    if face_dir.back_face {
        // Matches back face vertex order: (u0,v0), (u0,v1), (u1,v1), (u1,v0)
        uvs.push(uv_bl);
        uvs.push(uv_tl);
        uvs.push(uv_tr);
        uvs.push(uv_br);
    } else {
        // Matches front face vertex order: (u0,v0), (u1,v0), (u1,v1), (u0,v1)
        uvs.push(uv_bl);
        uvs.push(uv_br);
        uvs.push(uv_tr);
        uvs.push(uv_tl);
    }

    // UV_1: tile origin [u_min, v_min] — same for all 4 vertices of this quad.
    // The fragment shader uses this to wrap tiling UVs within the correct atlas tile.
    let [tile_u_min, tile_v_min, _, _] = tile_uvs(texture_index(block, face_dir.face));
    let tile_origin = [tile_u_min, tile_v_min];
    uv1s.push(tile_origin);
    uv1s.push(tile_origin);
    uv1s.push(tile_origin);
    uv1s.push(tile_origin);

    // Two triangles per quad — flip winding for faces where the default
    // u_axis/v_axis ordering produces a clockwise winding instead of CCW.
    if face_dir.flip_winding {
        indices.push(base_index);
        indices.push(base_index + 2);
        indices.push(base_index + 1);
        indices.push(base_index);
        indices.push(base_index + 3);
        indices.push(base_index + 2);
    } else {
        indices.push(base_index);
        indices.push(base_index + 1);
        indices.push(base_index + 2);
        indices.push(base_index);
        indices.push(base_index + 2);
        indices.push(base_index + 3);
    }
}

/// Emit a cross-billboard (X-shaped) mesh for non-cube blocks like torches, tallgrass, saplings.
/// Two diagonal quads intersecting at the block center, each rendered double-sided.
fn emit_cross_billboard(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    uv1s: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    bx: f32,
    by: f32,
    bz: f32,
    block: BlockType,
) {
    let tex_idx = texture_index(block, Face::South);
    let tile = tile_uvs(tex_idx);
    let tile_origin = [tile[0], tile[1]];

    // UV corners: bottom-left, bottom-right, top-right, top-left
    let uv_bl = [tile[0], tile[3]];
    let uv_br = [tile[2], tile[3]];
    let uv_tr = [tile[2], tile[1]];
    let uv_tl = [tile[0], tile[1]];

    // Inset so cross doesn't poke out of block boundaries
    let inset = 0.15;
    let lo = inset;
    let hi = 1.0 - inset;

    // Uniform upward normal for all billboard faces — gives even lighting from above
    let normal = [0.0, 1.0, 0.0_f32];

    // Quad 1 vertices: diagonal from (lo,0,lo) to (hi,1,hi)
    let q1 = [
        [bx + lo, by,       bz + lo], // bottom-left
        [bx + hi, by,       bz + hi], // bottom-right
        [bx + hi, by + 1.0, bz + hi], // top-right
        [bx + lo, by + 1.0, bz + lo], // top-left
    ];

    // Quad 2 vertices: diagonal from (hi,0,lo) to (lo,1,hi)
    let q2 = [
        [bx + hi, by,       bz + lo], // bottom-left
        [bx + lo, by,       bz + hi], // bottom-right
        [bx + lo, by + 1.0, bz + hi], // top-right
        [bx + hi, by + 1.0, bz + lo], // top-left
    ];

    let front_uvs = [uv_bl, uv_br, uv_tr, uv_tl];
    let back_uvs = [uv_br, uv_bl, uv_tl, uv_tr]; // horizontally mirrored for back face

    // Helper: emit one side of a quad (front or back)
    // Front face: 0,1,2, 0,2,3 (CCW)
    // Back face:  0,2,1, 0,3,2 (CW = reversed)
    let mut emit_side = |verts: &[[f32; 3]; 4], face_uvs: &[[f32; 2]; 4], front: bool| {
        let base = positions.len() as u32;
        for &v in verts {
            positions.push(v);
        }
        for _ in 0..4 {
            normals.push(normal);
            uv1s.push(tile_origin);
        }
        uvs.extend_from_slice(face_uvs);

        if front {
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        } else {
            indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }
    };

    // Quad 1: front and back
    emit_side(&q1, &front_uvs, true);
    emit_side(&q1, &back_uvs, false);

    // Quad 2: front and back
    emit_side(&q2, &front_uvs, true);
    emit_side(&q2, &back_uvs, false);
}

fn empty_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, Vec::<[f32; 2]>::new())
        .with_inserted_indices(Indices::U32(Vec::new()))
}
