use crate::voxel::block::{Block, BlockType};
use crate::voxel::direction::Direction;
use crate::voxel::texture::{convert_face_id_to_uv, UvCoordinate};
use bevy::math::{IVec3, Vec3};

pub struct Quad {
    pub direction: Direction,
    pub corners: [Vec3; 4],
    pub uvs: UvCoordinate,
}

pub const HALF_SIZE: f32 = 0.5f32;

impl Quad {
    pub fn from_direction(direction: Direction, i_pos: IVec3, voxel_type: BlockType) -> Self {
        let pos: Vec3 = i_pos.as_vec3();

        let corners = match direction {
            Direction::Left => [
                // -X face (Viewed from -X)
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE), // Top-Back-Left
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE), // Top-Front-Left
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE), // Bottom-Front-Left
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE), // Bottom-Back-Left
            ],
            Direction::Right => [
                // +X face (Viewed from +X)
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE), // Top-Front-Right
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE), // Top-Back-Right
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE), // Bottom-Back-Right
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE), // Bottom-Front-Right
            ],
            Direction::Down => [
                // -Y face (Viewed from -Y)
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE), // Back-Left-Bottom
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE), // Back-Right-Bottom
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE), // Front-Right-Bottom
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE), // Front-Left-Bottom
            ],
            Direction::Up => [
                // +Y face (Viewed from +Y)
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE), // Back-Left-Top (Vertex 0)
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE), // Back-Right-Top (Vertex 1)
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE), // Front-Right-Top (Vertex 2)
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE), // Front-Left-Top (Vertex 3)
            ],
            Direction::Back => [
                // -Z face (Viewed from -Z)
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE), // Top-Left-Back
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE), // Top-Right-Back
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE), // Bottom-Right-Back
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE), // Bottom-Left-Back
            ],
            Direction::Forward => [
                // +Z face (Viewed from +Z)
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE), // Top-Right-Front
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE), // Top-Left-Front
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE), // Bottom-Left-Front
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE), // Bottom-Right-Front
            ],
        };

        // UV coordinates are generated based on face_id
        let uvs = convert_face_id_to_uv(Block::get_face(&voxel_type, &direction));

        Self {
            corners,
            direction,
            uvs, // Standard UV order: [(minU,minV), (maxU,minV), (maxU,maxV), (minU,maxV)]
        }
    }
}
