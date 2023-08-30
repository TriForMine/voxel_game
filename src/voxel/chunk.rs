use crate::voxel::voxel::{Voxel, VoxelType};
use crate::voxel::world::World;
use bevy::math::IVec3;
use bevy::prelude::Component;
use std::sync::{RwLock, Weak};

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

#[derive(Clone, Debug)]
pub struct Chunk {
    pub voxels: ChunkData,
    pub pos: IVec3,
    neighbors: [Weak<RwLock<Chunk>>; 4],
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            voxels: [Voxel::new_empty(); (SIZE * SIZE * HEIGHT) as usize],
            pos: IVec3::default(),
            neighbors: [Weak::new(), Weak::new(), Weak::new(), Weak::new()],
        }
    }
}

impl Chunk {
    pub fn set_neighbor(&mut self, index: usize, chunk: Weak<RwLock<Chunk>>) {
        self.neighbors[index] = chunk;
    }

    pub fn get_index(coordinate: &IVec3) -> usize {
        (coordinate.z * SIZE * HEIGHT + coordinate.y * SIZE + coordinate.x) as usize
    }

    pub fn is_in_chunk(coordinate: &IVec3) -> bool {
        coordinate.y >= 0
            && coordinate.y < HEIGHT
            && coordinate.x >= 0
            && coordinate.x < SIZE
            && coordinate.z >= 0
            && coordinate.z < SIZE
    }

    pub fn get_voxel(&self, coordinate: IVec3) -> Option<Voxel> {
        if Self::is_in_chunk(&coordinate) {
            Some(self.voxels[Self::get_index(&coordinate)])
        } else if coordinate.x < 0 {
            // Left
            self.neighbors[0].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate + IVec3::new(SIZE, 0, 0)))]
            })
        } else if coordinate.x >= SIZE {
            // Right
            self.neighbors[1].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate - IVec3::new(SIZE, 0, 0)))]
            })
        } else if coordinate.z < 0 {
            // Back
            self.neighbors[2].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate + IVec3::new(0, 0, SIZE)))]
            })
        } else if coordinate.z >= SIZE {
            // Front
            self.neighbors[3].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate - IVec3::new(0, 0, SIZE)))]
            })
        } else {
            None
        }
    }

    pub fn edit_voxel(&mut self, world: &World, local_coordinate: IVec3, new_type: VoxelType) {
        if Self::is_in_chunk(&local_coordinate) {
            self.voxels[Self::get_index(&local_coordinate)].voxel_type = new_type;
            self.update_chunk(world);
            self.update_surrounding_voxels(world, local_coordinate);
        }
    }

    pub fn update_chunk(&mut self, world: &World) {
        let dirty_chunks = &world.dirty_chunks;
        dirty_chunks.write().unwrap().insert(self.pos);
    }

    pub fn update_surrounding_voxels(&mut self, world: &World, local_coordinate: IVec3) {
        if local_coordinate.x == 0 {
            self.update_neighbor(world, 0);
        } else if local_coordinate.x == SIZE - 1 {
            self.update_neighbor(world, 1);
        }

        if local_coordinate.z == 0 {
            self.update_neighbor(world, 2);
        } else if local_coordinate.z == SIZE - 1 {
            self.update_neighbor(world, 3);
        }
    }

    pub fn update_neighbor(&mut self, world: &World, index: usize) {
        if let Some(neighbor) = self.neighbors[index].upgrade() {
            let mut neighbor = neighbor.write().unwrap();
            neighbor.update_chunk(world);
        }
    }

    /// Get the neighbors of a voxel
    ///
    /// # Arguments
    ///
    /// * `coordinate` - The coordinate of the voxel
    ///
    /// # Returns
    ///
    /// An array of 6 options of voxels
    ///
    /// 0: right
    ///
    /// 1: left
    ///
    /// 2: top
    ///
    /// 3: bottom
    ///
    /// 4: front
    ///
    /// 5: back
    pub fn get_voxel_neighbors(&self, coordinate: IVec3) -> [Option<Voxel>; 6] {
        let mut neighbors = [None; 6];
        neighbors[0] = self.get_voxel(coordinate + IVec3::new(1, 0, 0));
        neighbors[1] = self.get_voxel(coordinate + IVec3::new(-1, 0, 0));
        neighbors[2] = self.get_voxel(coordinate + IVec3::new(0, 1, 0));
        neighbors[3] = self.get_voxel(coordinate + IVec3::new(0, -1, 0));
        neighbors[4] = self.get_voxel(coordinate + IVec3::new(0, 0, 1));
        neighbors[5] = self.get_voxel(coordinate + IVec3::new(0, 0, -1));
        neighbors
    }
}

#[derive(Component)]
pub struct ChunkEntity(pub IVec3);
