use crate::voxel::chunk::{Chunk, SIZE};
use crate::voxel::voxel::Voxel;
use bevy::asset::Handle;
use bevy::math::IVec3;
use bevy::prelude::{Component, Entity, Mesh};
use bevy::tasks::Task;
use dashmap::{DashMap, DashSet};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Component)]
pub struct ComputeMesh(pub Task<(Mesh, IVec3)>);

pub const DEFAULT_MAX_CHUNKS: usize = 10000;

pub type ChunkDataMap = Arc<Mutex<HashMap<IVec3, Chunk>>>;

pub struct World {
    pub(crate) chunk_data_map: ChunkDataMap,
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
        while local_pos.x > SIZE {
            local_pos.x -= SIZE;
            chunk_pos.x += 1;
        }
        while local_pos.z < 0 {
            local_pos.z += SIZE;
            chunk_pos.z -= 1;
        }
        while local_pos.z > SIZE {
            local_pos.z -= SIZE;
            chunk_pos.z += 1;
        }
    }
}
