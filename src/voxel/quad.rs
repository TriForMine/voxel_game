use bevy::math::Vec3;
use crate::flycam::prelude::IVec3;
use crate::voxel::direction::Direction;

pub struct Quad {
    pub direction: Direction,
    pub corners: [Vec3; 4]
}

const HALF_SIZE: f32 = 0.5f32;

impl Quad {
    pub fn from_direction(direction: Direction, i_pos: IVec3) -> Self {
        let pos: Vec3 = i_pos.as_vec3();

        let corners = match direction {
            Direction::Left => [
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Right => [
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Down => [
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
            ],
            Direction::Up => [
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Back => [
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Forward => [
                Vec3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                Vec3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
            ],
        };


        Self {
            corners,
            direction,
        }
    }
}