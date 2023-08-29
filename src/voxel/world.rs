use crate::voxel::chunk::{Chunk, HEIGHT, SIZE};
use crate::voxel::voxel::VoxelType;
use bevy::math::{IVec2, IVec3, Vec3};
use bevy::prelude::{Component, Entity, Mesh};
use bevy::tasks::Task;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Component)]
pub struct ComputeMesh(pub Task<(Mesh, IVec3)>);

pub const DEFAULT_MAX_CHUNKS: usize = 10000;

pub type ChunkDataMap = HashMap<IVec3, Chunk>;

pub struct World {
    pub(crate) chunk_data_map: Arc<Mutex<ChunkDataMap>>,
    pub(crate) chunk_entities: Arc<Mutex<HashMap<IVec3, Entity>>>,
    pub(crate) dirty_chunks: Arc<Mutex<HashSet<IVec3>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunk_data_map: Arc::new(Mutex::new(HashMap::with_capacity(DEFAULT_MAX_CHUNKS))),
            chunk_entities: Arc::new(Mutex::new(HashMap::with_capacity(DEFAULT_MAX_CHUNKS))),
            dirty_chunks: Arc::new(Mutex::new(HashSet::with_capacity(DEFAULT_MAX_CHUNKS))),
        }
    }

    pub fn make_coords_valid(chunk_pos: &mut IVec3, local_pos: &mut IVec3) {
        while local_pos.x < 0 {
            local_pos.x += SIZE;
            chunk_pos.x -= 1;
        }
        while local_pos.x >= SIZE {
            local_pos.x -= SIZE;
            chunk_pos.x += 1;
        }
        while local_pos.z < 0 {
            local_pos.z += SIZE;
            chunk_pos.z -= 1;
        }
        while local_pos.z >= SIZE {
            local_pos.z -= SIZE;
            chunk_pos.z += 1;
        }
    }

    pub fn check_block_at_coord(&self, global_coord: &IVec3) -> bool {
        let mut chunk_coord = IVec3::default();
        let mut local_coord = *global_coord;
        Self::make_coords_valid(&mut chunk_coord, &mut local_coord);
        let chunks = self.chunk_data_map.lock().unwrap();
        let chunk = chunks.get(&chunk_coord);

        if let Some(chunk) = chunk {
            chunk.voxels[Chunk::get_index(&local_coord)].voxel_type != VoxelType::Void
        } else {
            false
        }
    }

    pub fn get_highest_block_at_coord(&self, global_coord: &IVec2) -> IVec3 {
        let mut chunk_coord = IVec3::default();
        let mut local_coord = IVec3::new(global_coord.x, HEIGHT - 1, global_coord.y);
        Self::make_coords_valid(&mut chunk_coord, &mut local_coord);
        let chunks = self.chunk_data_map.lock().unwrap();
        let chunk = chunks.get(&chunk_coord);

        if let Some(chunk) = chunk {
            while local_coord.y > 0
                && chunk.voxels[Chunk::get_index(&local_coord)].voxel_type == VoxelType::Void
            {
                local_coord.y -= 1;
            }
        } else {
            todo!("Force load chunk, to get the height");
        };

        Self::chunk_local_to_world(&chunk_coord, &local_coord)
    }

    pub fn coord_to_chunk_local(origin: Vec3) -> IVec3 {
        IVec3::new(
            ((origin.x + 0.5).floor() as i32) % SIZE,
            origin.y.floor() as i32,
            ((origin.z + 0.5).floor() as i32) % SIZE,
        )
    }

    pub fn coord_to_world(origin: Vec3) -> IVec3 {
        IVec3::new(
            (origin.x + 0.5).floor() as i32,
            origin.y.floor() as i32,
            (origin.z + 0.5).floor() as i32,
        )
    }

    pub fn chunk_local_to_world(chunk_coord: &IVec3, voxel_coord: &IVec3) -> IVec3 {
        IVec3::new(
            chunk_coord.x * SIZE + voxel_coord.x,
            voxel_coord.y,
            chunk_coord.z * SIZE + voxel_coord.z,
        )
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_make_coords_valid_same_chunk() {
        let mut chunk_pos = IVec3::new(0, 0, 0);
        let mut local_pos = IVec3::new(5, 75, 5);

        World::make_coords_valid(&mut chunk_pos, &mut local_pos);

        assert_eq!(chunk_pos, IVec3::new(0, 0, 0));
        assert_eq!(local_pos, IVec3::new(5, 75, 5));
    }

    #[test]
    fn test_make_coords_valid_neighbour_chunk() {
        let mut chunk_pos = IVec3::new(0, 0, 0);
        let mut local_pos = IVec3::new(-1, 75, 5);

        World::make_coords_valid(&mut chunk_pos, &mut local_pos);

        assert_eq!(chunk_pos, IVec3::new(-1, 0, 0));
        assert_eq!(local_pos, IVec3::new(15, 75, 5));
    }

    #[test]
    fn test_make_coords_valid_neighbour_chunk2() {
        let mut chunk_pos = IVec3::new(0, 0, 0);
        let mut local_pos = IVec3::new(16, 75, 5);

        World::make_coords_valid(&mut chunk_pos, &mut local_pos);

        assert_eq!(chunk_pos, IVec3::new(1, 0, 0));
        assert_eq!(local_pos, IVec3::new(0, 75, 5));
    }
}
