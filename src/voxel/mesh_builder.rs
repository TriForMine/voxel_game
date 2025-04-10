// src/voxel/mesh_builder.rs

use crate::chunk::{CHUNK_HEIGHT, CHUNK_SIZE};
use crate::voxel::block::Block; // Make sure BlockType is imported
use crate::voxel::chunk::{Chunk, ChunkData}; // Make sure ChunkData is imported
use crate::voxel::direction::Direction;
use crate::voxel::texture::convert_face_id_to_uv; // Keep this
use bevy::asset::RenderAssetUsages;
use bevy::math::IVec3;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::PrimitiveTopology;
use std::sync::RwLockReadGuard;
use std::time::Instant;

// Precompute corner offsets for each face direction relative to voxel center (0,0,0)
// Order: Top-Left, Top-Right, Bottom-Right, Bottom-Left (relative to viewing the face)
// Matches the typical quad vertex order for triangulation (0, 1, 2, 0, 2, 3)
const FACE_CORNERS: [[Vec3; 4]; 6] = [
    // Right (+X face)
    [
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, -0.5),
    ],
    // Left (-X face)
    [
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
    ],
    // Up (+Y face) - Top-Left from above is Back-Left
    [
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(-0.5, 0.5, -0.5),
    ],
    // Down (-Y face) - Top-Left from below is Front-Left
    [
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),
    ],
    // Forward (+Z face)
    [
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, -0.5, 0.5),
        Vec3::new(0.5, -0.5, 0.5),
    ],
    // Back (-Z face)
    [
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5),
    ],
];

// Normals for each face direction
const FACE_NORMALS: [Vec3; 6] = [
    Vec3::X,
    Vec3::NEG_X,
    Vec3::Y,
    Vec3::NEG_Y,
    Vec3::Z,
    Vec3::NEG_Z,
];

// Holds optional read guards for neighbor chunks
struct NeighborGuards<'a> {
    left: Option<RwLockReadGuard<'a, Chunk>>,
    right: Option<RwLockReadGuard<'a, Chunk>>,
    back: Option<RwLockReadGuard<'a, Chunk>>,
    forward: Option<RwLockReadGuard<'a, Chunk>>,
}

pub fn create_chunk_mesh(chunk: &Chunk) -> Mesh {
    // --- Start Timing ---
    let start_time = Instant::now();

    // --- Neighbor Arc Acquisition ---
    // Get the Arcs first. They need to live until neighbor_guards goes out of scope.
    let neighbor_left_arc_opt = chunk.neighbors[0].upgrade();
    let neighbor_right_arc_opt = chunk.neighbors[1].upgrade();
    let neighbor_back_arc_opt = chunk.neighbors[2].upgrade();
    let neighbor_forward_arc_opt = chunk.neighbors[3].upgrade();

    // --- Neighbor Lock Acquisition ---
    // Now create the guards, borrowing from the Arcs above.
    // Use .as_ref() to borrow the Arc from the Option before calling read().
    let neighbor_guards = NeighborGuards {
        left: neighbor_left_arc_opt
            .as_ref()
            .and_then(|arc| arc.read().ok()),
        right: neighbor_right_arc_opt
            .as_ref()
            .and_then(|arc| arc.read().ok()),
        back: neighbor_back_arc_opt
            .as_ref()
            .and_then(|arc| arc.read().ok()),
        forward: neighbor_forward_arc_opt
            .as_ref()
            .and_then(|arc| arc.read().ok()),
    };

    // --- Mesh Data Initialization ---
    // Estimate capacity: Max possible quads is CHUNK_SIZE*CHUNK_SIZE*CHUNK_HEIGHT*6, but reality is much less.
    // A rough estimate (e.g., 1/4th of voxels have 3 exposed faces) might be okay.
    // Let's estimate based on potential surface area + some internal faces.
    let estimated_quads = (CHUNK_SIZE * CHUNK_SIZE * 3)
        + (CHUNK_SIZE * CHUNK_HEIGHT * 3)
        + (CHUNK_SIZE * CHUNK_HEIGHT * 3); // Rough estimate
    let estimated_vertices = estimated_quads * 4;
    let estimated_indices = estimated_quads * 6;

    let mut vertices = Vec::<Vec3>::with_capacity(estimated_vertices as usize);
    let mut normals = Vec::<Vec3>::with_capacity(estimated_vertices as usize);
    let mut uvs = Vec::<Vec2>::with_capacity(estimated_vertices as usize);
    let mut indices = Vec::<u32>::with_capacity(estimated_indices as usize);
    let mut current_vertex_index: u32 = 0;

    let chunk_voxels = &chunk.voxels; // Borrow voxel data locally

    // --- Main Meshing Loop ---
    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let voxel_pos_local = IVec3::new(x, y, z);
                let voxel_index = Chunk::get_index(&voxel_pos_local);
                let current_voxel = chunk_voxels[voxel_index];

                if !current_voxel.is_solid() {
                    continue; // Skip air blocks
                }

                let current_voxel_type = current_voxel.voxel_type;
                let current_voxel_world_pos = voxel_pos_local.as_vec3(); // For positioning quads

                // --- Neighbor Check and Quad Generation ---
                // Iterate through 6 directions (Right, Left, Up, Down, Forward, Back)
                for direction_index in 0..6 {
                    let neighbor_voxel = get_voxel_neighbor_optimized(
                        voxel_pos_local,
                        direction_index,
                        chunk_voxels,     // Pass current chunk's data
                        &neighbor_guards, // Pass neighbor guards
                    );

                    if should_add_face(neighbor_voxel) {
                        // Add face directly to mesh data vectors
                        let face_normal = FACE_NORMALS[direction_index];
                        let texture_coords = convert_face_id_to_uv(Block::get_face(
                            &current_voxel_type,
                            &Direction::from_index(direction_index),
                        ));

                        for (i, texture_coord) in texture_coords.iter().enumerate() {
                            // Calculate vertex position relative to chunk origin
                            vertices
                                .push(current_voxel_world_pos + FACE_CORNERS[direction_index][i]);
                            normals.push(face_normal);
                            uvs.push(*texture_coord); // Use UVs from texture atlas lookup
                        }

                        // Add indices for the two triangles forming the quad
                        indices.push(current_vertex_index); // Triangle 1: Vertex 0
                        indices.push(current_vertex_index + 1); // Triangle 1: Vertex 1
                        indices.push(current_vertex_index + 2); // Triangle 1: Vertex 2

                        indices.push(current_vertex_index); // Triangle 2: Vertex 0
                        indices.push(current_vertex_index + 2); // Triangle 2: Vertex 2
                        indices.push(current_vertex_index + 3); // Triangle 2: Vertex 3

                        current_vertex_index += 4;
                    }
                }
            }
        }
    }

    // --- Final Mesh Construction ---
    let mut chunk_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    chunk_mesh.insert_indices(Indices::U32(indices));

    // --- End Timing & Log ---
    let elapsed = start_time.elapsed(); // <-- Calculate elapsed time
                                        // Log using Bevy's debug macro. Includes chunk position for context.
    debug!(
        "Mesh generation for chunk {:?} took {:?}",
        chunk.pos, elapsed
    ); // <-- Log the duration

    chunk_mesh
}

// Optimized neighbor lookup using pre-acquired locks (guards)
#[inline]
fn get_voxel_neighbor_optimized<'a>(
    voxel_pos: IVec3,                        // Local position in the current chunk
    direction_index: usize, // 0..5 corresponding to Right, Left, Up, Down, Forward, Back
    current_chunk_voxels: &'a ChunkData, // Voxel data of the chunk being meshed
    neighbor_guards: &'a NeighborGuards<'a>, // Locked neighbor data
) -> Option<&'a Block> {
    // Return Option<&Block> to match should_add_face

    match direction_index {
        // --- X Axis ---
        0 => {
            // Right (+X)
            if voxel_pos.x + 1 >= CHUNK_SIZE {
                // Check Right Neighbor Chunk
                neighbor_guards.right.as_ref().map(|guard| {
                    let neighbor_local_pos = IVec3::new(0, voxel_pos.y, voxel_pos.z);
                    &guard.voxels[Chunk::get_index(&neighbor_local_pos)]
                })
            } else {
                // Within current chunk
                Some(&current_chunk_voxels[Chunk::get_index(&(voxel_pos + IVec3::X))])
            }
        }
        1 => {
            // Left (-X)
            if voxel_pos.x - 1 < 0 {
                // Check Left Neighbor Chunk
                neighbor_guards.left.as_ref().map(|guard| {
                    let neighbor_local_pos = IVec3::new(CHUNK_SIZE - 1, voxel_pos.y, voxel_pos.z);
                    &guard.voxels[Chunk::get_index(&neighbor_local_pos)]
                })
            } else {
                // Within current chunk
                Some(&current_chunk_voxels[Chunk::get_index(&(voxel_pos - IVec3::X))])
            }
        }
        // --- Y Axis ---
        2 => {
            // Up (+Y)
            if voxel_pos.y + 1 >= CHUNK_HEIGHT {
                None
            }
            // Above world
            else {
                Some(&current_chunk_voxels[Chunk::get_index(&(voxel_pos + IVec3::Y))])
            }
        }
        3 => {
            // Down (-Y)
            if voxel_pos.y - 1 < 0 {
                None
            }
            // Below world
            else {
                Some(&current_chunk_voxels[Chunk::get_index(&(voxel_pos - IVec3::Y))])
            }
        }
        // --- Z Axis ---
        4 => {
            // Forward (+Z)
            if voxel_pos.z + 1 >= CHUNK_SIZE {
                // Check Forward Neighbor Chunk
                neighbor_guards.forward.as_ref().map(|guard| {
                    let neighbor_local_pos = IVec3::new(voxel_pos.x, voxel_pos.y, 0);
                    &guard.voxels[Chunk::get_index(&neighbor_local_pos)]
                })
            } else {
                // Within current chunk
                Some(&current_chunk_voxels[Chunk::get_index(&(voxel_pos + IVec3::Z))])
            }
        }
        5 => {
            // Back (-Z)
            if voxel_pos.z - 1 < 0 {
                // Check Back Neighbor Chunk
                neighbor_guards.back.as_ref().map(|guard| {
                    let neighbor_local_pos = IVec3::new(voxel_pos.x, voxel_pos.y, CHUNK_SIZE - 1);
                    &guard.voxels[Chunk::get_index(&neighbor_local_pos)]
                })
            } else {
                // Within current chunk
                Some(&current_chunk_voxels[Chunk::get_index(&(voxel_pos - IVec3::Z))])
            }
        }
        _ => unreachable!(), // Should be 0..5
    }
}

#[inline]
fn should_add_face(neighbor_voxel: Option<&Block>) -> bool {
    match neighbor_voxel {
        Some(voxel) => !voxel.is_solid(), // Add face if neighbor is not solid (e.g., air)
        None => true, // Add face if neighbor is outside the loaded chunk or world bounds
    }
}
