use crate::voxel::voxel::{Voxel, VoxelType};
use bevy::math::IVec3;
use bevy::prelude::Component;

use lazy_static::*;
lazy_static! {
    // when SIZE 16, BIT_SIZE is 4
    // by shifting 16 << 4 we get 1
    // we with this get indexes from the collapsed array
    pub static ref BIT_SIZE: i32 = (SIZE as f32).log2() as i32;
    pub static ref BIT_SIZE_HEIGHT: i32 = (HEIGHT as f32).log2() as i32;
}

pub const SIZE: i32 = 16;
pub const HEIGHT: i32 = 256;

pub type ChunkData = [Voxel; (SIZE * SIZE * HEIGHT) as usize];

#[derive(Clone, Copy)]
pub struct Chunk {
    pub voxels: ChunkData,
}

impl Chunk {
    pub fn get_index(coordinate: &IVec3) -> usize {
        (coordinate.z * SIZE * HEIGHT + coordinate.y * SIZE + coordinate.x) as usize
    }

    pub fn get_local_coordinate(index: i32) -> IVec3 {
        let z = index / SIZE * HEIGHT;
        let y = (index - z * SIZE * HEIGHT) / SIZE;
        let x = index - z * SIZE * HEIGHT - y * SIZE;
        IVec3::new(x, y, z)
    }

    pub fn is_outside_chunk(coordinate: &IVec3) -> bool {
        coordinate.y < 0
            || coordinate.y >= HEIGHT
            || coordinate.x < 0
            || coordinate.x >= SIZE
            || coordinate.z < 0
            || coordinate.z >= SIZE
    }

    pub fn get_voxel(&self, coordinate: IVec3) -> Option<&Voxel> {
        if Self::is_outside_chunk(&coordinate) {
            return None;
        }

        let index = Self::get_index(&coordinate);
        self.get_voxel_from_index(index)
    }

    pub fn get_mut_voxel(&mut self, coordinate: &IVec3) -> Option<&mut Voxel> {
        if Self::is_outside_chunk(&coordinate) {
            return None;
        }

        let index = Self::get_index(&coordinate);
        self.get_mut_voxel_from_index(index)
    }

    pub fn get_voxel_from_index(&self, index: usize) -> Option<&Voxel> {
        self.voxels.get(index)
    }

    pub fn get_mut_voxel_from_index(&mut self, index: usize) -> Option<&mut Voxel> {
        self.voxels.get_mut(index)
    }

    pub fn new() -> Self {
        let chunk = Self {
            voxels: [Voxel::new_empty(); (SIZE * SIZE * HEIGHT) as usize],
        };
        chunk
    }

    pub fn new_from_voxels(voxels: ChunkData) -> Self {
        Self { voxels }
    }

    fn reset(&mut self) {
        for voxel in self.voxels.iter_mut() {
            voxel.set_type(VoxelType::Void)
        }
    }
}

#[derive(Component)]
pub struct ChunkEntity(pub IVec3);
