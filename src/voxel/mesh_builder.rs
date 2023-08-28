use super::chunk::HEIGHT;
use super::chunk::SIZE;
use crate::voxel::direction::Direction;
use crate::voxel::quad::Quad;
use crate::voxel::voxel::Voxel;
use crate::voxel::world::{ChunkDataMap, World};
use crate::WORLD_SIZE;
use anyhow::*;
use bevy::math::IVec3;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::PrimitiveTopology;
use std::time::Instant;

pub fn create_chunk_mesh(chunk_data_map: &ChunkDataMap, chunk_pos: &IVec3) -> Mesh {
    let start = Instant::now();
    let mut chunk_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut quads = Vec::<Quad>::new();

    for x in 0..(SIZE) {
        for z in 0..(SIZE) {
            for y in 0..(HEIGHT) {
                let voxel_pos_local = IVec3::new(x, y, z);

                if let Result::Ok((voxel, front, back, left, right, top, down)) =
                    adjacent_voxels(chunk_data_map, chunk_pos, &voxel_pos_local)
                {
                    process_voxel(
                        voxel.as_ref(),
                        voxel_pos_local,
                        front.as_ref(),
                        back.as_ref(),
                        left.as_ref(),
                        right.as_ref(),
                        top.as_ref(),
                        down.as_ref(),
                        &mut quads,
                    );
                }
            }
        }
    }

    let mut vertices = Vec::<Vec3>::with_capacity(quads.len() * 4);
    let mut normals = Vec::<Vec3>::with_capacity(quads.len() * 4);
    let mut uvs = Vec::<Vec2>::with_capacity(quads.len() * 4);
    let mut indices = Vec::<u32>::with_capacity(quads.len() * 6);
    let mut vert_index = 0;

    for quad in quads {
        let normal = quad.direction.get_normal();
        vertices.extend_from_slice(&quad.corners);
        uvs.extend_from_slice(&quad.uvs);

        (0..4).for_each(|_| {
            normals.push(normal);
        });

        indices.push(vert_index);
        indices.push(vert_index + 1);
        indices.push(vert_index + 2);
        indices.push(vert_index);
        indices.push(vert_index + 2);
        indices.push(vert_index + 3);
        vert_index += 4;
    }

    let duration = start.elapsed();

    trace!(
        "Chunk vertices and indices generated in: {:?} for: {:?}",
        duration,
        chunk_pos
    );

    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    chunk_mesh.set_indices(Some(Indices::U32(indices)));

    chunk_mesh
}

fn process_voxel(
    voxel: Option<&Voxel>,
    voxel_pos: IVec3,
    front: Option<&Voxel>,
    back: Option<&Voxel>,
    left: Option<&Voxel>,
    right: Option<&Voxel>,
    top: Option<&Voxel>,
    down: Option<&Voxel>,
    quads: &mut Vec<Quad>,
) {
    if let Some(voxel) = voxel {
        if voxel.is_solid() {
            if left.is_none()
                || match left {
                    Some(left) => !left.is_solid(),
                    None => true,
                }
            {
                quads.push(Quad::from_direction(
                    Direction::Left,
                    voxel_pos,
                    voxel.voxel_type,
                ))
            }
            if right.is_none()
                || match right {
                    Some(right) => !right.is_solid(),
                    None => true,
                }
            {
                quads.push(Quad::from_direction(
                    Direction::Right,
                    voxel_pos,
                    voxel.voxel_type,
                ))
            }
            if top.is_none()
                || match top {
                    Some(top) => !top.is_solid(),
                    None => true,
                }
            {
                quads.push(Quad::from_direction(
                    Direction::Up,
                    voxel_pos,
                    voxel.voxel_type,
                ))
            }
            if down.is_none()
                || match down {
                    Some(right) => !right.is_solid(),
                    None => true,
                }
            {
                quads.push(Quad::from_direction(
                    Direction::Down,
                    voxel_pos,
                    voxel.voxel_type,
                ))
            }
            if front.is_none()
                || match front {
                    Some(front) => !front.is_solid(),
                    None => true,
                }
            {
                quads.push(Quad::from_direction(
                    Direction::Forward,
                    voxel_pos,
                    voxel.voxel_type,
                ))
            }
            if back.is_none()
                || match back {
                    Some(back) => !back.is_solid(),
                    None => true,
                }
            {
                quads.push(Quad::from_direction(
                    Direction::Back,
                    voxel_pos,
                    voxel.voxel_type,
                ))
            }
        };
    };
}

fn try_get_voxel(
    chunk_data_map: &ChunkDataMap,
    chunk_pos: &IVec3,
    local_pos: &IVec3,
) -> Option<Voxel> {
    let mut chunk_pos = *chunk_pos;
    let mut local_pos = *local_pos;
    World::make_coords_valid(&mut chunk_pos, &mut local_pos);

    if chunk_pos.x < -WORLD_SIZE
        || chunk_pos.x > WORLD_SIZE
        || chunk_pos.z < -WORLD_SIZE
        || chunk_pos.z > WORLD_SIZE
    {
        println!("Chunk Pos: {:?} outside of the world", chunk_pos);
        return Some(Voxel::new_empty());
    }

    let chunk = chunk_data_map.get(&chunk_pos);

    if let Some(chunk) = chunk {
        chunk.get_voxel(local_pos).copied()
    } else {
        None
    }
}

pub fn adjacent_voxels(
    chunk_data_map: &ChunkDataMap,
    chunk_pos: &IVec3,
    local_pos: &IVec3,
) -> Result<(
    Option<Voxel>,
    Option<Voxel>,
    Option<Voxel>,
    Option<Voxel>,
    Option<Voxel>,
    Option<Voxel>,
    Option<Voxel>,
)> {
    let voxel = try_get_voxel(chunk_data_map, chunk_pos, local_pos);

    let front = try_get_voxel(
        chunk_data_map,
        chunk_pos,
        &(*local_pos + IVec3::new(0, 0, 1)),
    );

    let back = try_get_voxel(
        chunk_data_map,
        chunk_pos,
        &(*local_pos + IVec3::new(0, 0, -1)),
    );

    let left = try_get_voxel(
        chunk_data_map,
        chunk_pos,
        &(*local_pos + IVec3::new(-1, 0, 0)),
    );
    let right = try_get_voxel(
        chunk_data_map,
        chunk_pos,
        &(*local_pos + IVec3::new(1, 0, 0)),
    );

    let top = try_get_voxel(
        chunk_data_map,
        chunk_pos,
        &(*local_pos + IVec3::new(0, 1, 0)),
    );
    let down = try_get_voxel(
        chunk_data_map,
        chunk_pos,
        &(*local_pos + IVec3::new(0, -1, 0)),
    );

    Ok((voxel, front, back, left, right, top, down))
}
