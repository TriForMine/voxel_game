use crate::meshing::check_loading_world_ended;
use crate::terrain::chunk_generation::TerrainGenSet;
use crate::terrain::chunk_generation::{process_chunk_generation, queue_chunk_generation};
use crate::terrain::meshing::{
    check_server_loading_world_ended, clear_dirty_chunks, prepare_chunks, process_mesh_tasks,
    queue_mesh_tasks, ChunkMeshingSet,
};
use crate::voxel::block::{Block, BlockType};
use crate::voxel::world::World;
use crate::{ClientState, ServerState};
use bevy::math::IVec3;
use bevy::prelude::*;
use bincode::config;
use lazy_static::*;
use lz4::block::{compress, decompress, CompressionMode};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::sync::{RwLock, Weak};

lazy_static! {
    // when SIZE 16, BIT_SIZE is 4
    // by shifting 16 << 4 we get 1
    // we with this get indexes from the collapsed array
    pub static ref BIT_SIZE: i32 = (CHUNK_SIZE as f32).log2() as i32;
    pub static ref BIT_SIZE_HEIGHT: i32 = (CHUNK_HEIGHT as f32).log2() as i32;
}

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 256;

pub type CompressedChunk = Vec<u8>;
pub type ChunkData = [Block; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    #[serde(with = "BigArray")]
    pub voxels: ChunkData,
    pub pos: IVec3,

    #[serde(skip)]
    pub neighbors: [Weak<RwLock<Chunk>>; 4],
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            voxels: [Block::new_empty(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize],
            pos: IVec3::default(),
            neighbors: [Weak::new(), Weak::new(), Weak::new(), Weak::new()],
        }
    }
}

impl Chunk {
    pub fn from_compressed(bytes: &CompressedChunk) -> Self {
        let decompressed = decompress(bytes, None).unwrap();

        bincode::serde::decode_from_slice(&decompressed, config::standard())
            .unwrap()
            .0
    }

    pub fn compress(&self) -> CompressedChunk {
        let data = bincode::serde::encode_to_vec(self, config::standard()).unwrap();

        compress(&data, Some(CompressionMode::HIGHCOMPRESSION(12)), true).unwrap()
    }

    pub fn set_neighbor(&mut self, index: usize, chunk: Weak<RwLock<Chunk>>) {
        self.neighbors[index] = chunk;
    }

    pub fn get_index(coordinate: &IVec3) -> usize {
        (coordinate.z * CHUNK_SIZE * CHUNK_HEIGHT + coordinate.y * CHUNK_SIZE + coordinate.x)
            as usize
    }

    pub fn is_in_chunk(coordinate: &IVec3) -> bool {
        coordinate.y >= 0
            && coordinate.y < CHUNK_HEIGHT
            && coordinate.x >= 0
            && coordinate.x < CHUNK_SIZE
            && coordinate.z >= 0
            && coordinate.z < CHUNK_SIZE
    }

    pub fn get_voxel(&self, coordinate: IVec3) -> Option<Block> {
        if Self::is_in_chunk(&coordinate) {
            Some(self.voxels[Self::get_index(&coordinate)])
        } else if coordinate.x < 0 {
            // Left
            self.neighbors[0].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate + IVec3::new(CHUNK_SIZE, 0, 0)))]
            })
        } else if coordinate.x >= CHUNK_SIZE {
            // Right
            self.neighbors[1].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate - IVec3::new(CHUNK_SIZE, 0, 0)))]
            })
        } else if coordinate.z < 0 {
            // Back
            self.neighbors[2].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate + IVec3::new(0, 0, CHUNK_SIZE)))]
            })
        } else if coordinate.z >= CHUNK_SIZE {
            // Front
            self.neighbors[3].upgrade().map(|chunk| {
                chunk.read().unwrap().voxels
                    [Self::get_index(&(coordinate - IVec3::new(0, 0, CHUNK_SIZE)))]
            })
        } else {
            None
        }
    }

    pub fn edit_voxel(&mut self, world: &World, local_coordinate: IVec3, new_type: BlockType) {
        if Self::is_in_chunk(&local_coordinate)
            && self.voxels[Self::get_index(&local_coordinate)].voxel_type != new_type
        {
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
        } else if local_coordinate.x == CHUNK_SIZE - 1 {
            self.update_neighbor(world, 1);
        }

        if local_coordinate.z == 0 {
            self.update_neighbor(world, 2);
        } else if local_coordinate.z == CHUNK_SIZE - 1 {
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
    pub fn get_voxel_neighbors(&self, coordinate: IVec3) -> [Option<Block>; 6] {
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

#[derive(Component)]
pub struct ServerChunkEntity(pub IVec3);

pub struct ClientChunkPlugin;
impl Plugin for ClientChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            check_loading_world_ended.run_if(in_state(ClientState::LoadingWorld)),
        )
        .add_systems(
            Update,
            (prepare_chunks, queue_mesh_tasks, process_mesh_tasks)
                .chain()
                .in_set(ChunkMeshingSet)
                .run_if(in_state(ClientState::LoadingWorld).or(in_state(ClientState::Playing))),
        )
        .add_systems(
            Last,
            clear_dirty_chunks
                .run_if(in_state(ClientState::LoadingWorld).or(in_state(ClientState::Playing))),
        );
    }
}

pub struct ServerChunkPlugin;
impl Plugin for ServerChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            check_server_loading_world_ended.run_if(in_state(ServerState::LoadingWorld)),
        )
        .add_systems(
            Update,
            (queue_chunk_generation, process_chunk_generation)
                .chain()
                .in_set(TerrainGenSet)
                .run_if(in_state(ServerState::LoadingWorld).or(in_state(ServerState::Running))),
        );
    }
}
