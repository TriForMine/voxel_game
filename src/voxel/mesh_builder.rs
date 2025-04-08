use super::chunk::HEIGHT;
use super::chunk::SIZE;
use crate::voxel::block::Block;
use crate::voxel::chunk::Chunk;
use crate::voxel::direction::Direction;
use crate::voxel::quad::Quad;
use bevy::asset::RenderAssetUsages;
use bevy::math::IVec3;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::PrimitiveTopology;

pub fn create_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut chunk_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let mut quads = Vec::<Quad>::new();

    for x in 0..(SIZE) {
        for z in 0..(SIZE) {
            for y in 0..(HEIGHT) {
                let voxel_pos_local = IVec3::new(x, y, z);
                let voxel = chunk.get_voxel(voxel_pos_local);

                let [right, left, top, down, front, back] =
                    chunk.get_voxel_neighbors(voxel_pos_local);

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

    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    chunk_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    chunk_mesh.insert_indices(Indices::U32(indices));

    chunk_mesh
}

fn process_voxel(
    voxel: Option<&Block>,
    voxel_pos: IVec3,
    front: Option<&Block>,
    back: Option<&Block>,
    left: Option<&Block>,
    right: Option<&Block>,
    top: Option<&Block>,
    down: Option<&Block>,
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
