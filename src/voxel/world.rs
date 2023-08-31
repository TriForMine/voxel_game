use crate::voxel::block::{Block, BlockType};
use crate::voxel::chunk::ChunkEntity;
use crate::voxel::chunk::{Chunk, HEIGHT, SIZE};
use crate::ClientState;
use bevy::app::App;
use bevy::math::{IVec2, IVec3, Vec3};
use bevy::prelude::{
    default, Commands, Component, Entity, Mesh, OnEnter, Plugin, PointLight, PointLightBundle, Res,
    Resource, Transform,
};
use bevy::tasks::Task;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Weak};

#[derive(Component)]
pub struct ComputeMesh(pub Task<(Mesh, IVec3)>);

pub const DEFAULT_MAX_CHUNKS: usize = 10000;
pub const WORLD_SIZE: i32 = 5;

#[derive(Resource)]
pub struct GameWorld {
    pub world: Arc<RwLock<World>>,
}

impl Default for GameWorld {
    fn default() -> Self {
        Self {
            world: Arc::new(RwLock::new(World::new())),
        }
    }
}

pub type ChunkDataMap = HashMap<IVec3, Arc<RwLock<Chunk>>>;

pub struct World {
    pub(crate) chunk_data_map: Arc<RwLock<ChunkDataMap>>,
    pub(crate) chunk_entities: Arc<RwLock<HashMap<IVec3, Entity>>>,
    pub(crate) dirty_chunks: Arc<RwLock<HashSet<IVec3>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunk_data_map: Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_MAX_CHUNKS))),
            chunk_entities: Arc::new(RwLock::new(HashMap::with_capacity(DEFAULT_MAX_CHUNKS))),
            dirty_chunks: Arc::new(RwLock::new(HashSet::with_capacity(DEFAULT_MAX_CHUNKS))),
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

    pub fn get_voxel(&self, global_coord: &IVec3) -> Option<Block> {
        let mut chunk_coord = IVec3::default();
        let mut local_coord = *global_coord;
        Self::make_coords_valid(&mut chunk_coord, &mut local_coord);
        let chunks = self.chunk_data_map.read().unwrap();
        let chunk = chunks.get(&chunk_coord);

        if let Some(chunk) = chunk {
            chunk.read().unwrap().get_voxel(local_coord)
        } else {
            None
        }
    }

    pub fn edit_voxel(&self, global_coord: &IVec3, voxel_type: BlockType) {
        let mut chunk_coord = IVec3::default();
        let mut local_coord = *global_coord;
        Self::make_coords_valid(&mut chunk_coord, &mut local_coord);
        let chunks = self.chunk_data_map.read().unwrap();
        let chunk = chunks.get(&chunk_coord);

        if let Some(chunk) = chunk {
            chunk
                .write()
                .unwrap()
                .edit_voxel(self, local_coord, voxel_type);
        }
    }

    pub fn check_block_at_coord(&self, global_coord: &IVec3) -> bool {
        if let Some(voxel) = self.get_voxel(global_coord) {
            voxel.voxel_type != BlockType::Void
        } else {
            false
        }
    }

    pub fn get_highest_block_at_coord(&self, global_coord: &IVec2) -> IVec3 {
        let mut chunk_coord = IVec3::default();
        let mut local_coord = IVec3::new(global_coord.x, HEIGHT - 1, global_coord.y);
        Self::make_coords_valid(&mut chunk_coord, &mut local_coord);
        let chunks = self.chunk_data_map.read().unwrap();
        let chunk = chunks.get(&chunk_coord);

        if let Some(chunk) = chunk {
            while local_coord.y > 0
                && chunk.read().unwrap().voxels[Chunk::get_index(&local_coord)].voxel_type
                    == BlockType::Void
            {
                local_coord.y -= 1;
            }
        } else {
            todo!("Force load chunk, to get the height");
        };

        Self::chunk_local_to_world(&chunk_coord, &local_coord)
    }

    pub fn coord_to_world(origin: Vec3) -> IVec3 {
        IVec3::new(
            (origin.x + 0.5).floor() as i32,
            (origin.y + 0.5).floor() as i32,
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

    pub fn get_neighbors_chunks(&self, chunk_coord: &IVec3) -> [Option<Weak<RwLock<Chunk>>>; 4] {
        [
            self.chunk_data_map
                .read()
                .unwrap()
                .get(&IVec3::new(chunk_coord.x - 1, chunk_coord.y, chunk_coord.z))
                .map(|chunk| Arc::downgrade(chunk)),
            self.chunk_data_map
                .read()
                .unwrap()
                .get(&IVec3::new(chunk_coord.x + 1, chunk_coord.y, chunk_coord.z))
                .map(|chunk| Arc::downgrade(chunk)),
            self.chunk_data_map
                .read()
                .unwrap()
                .get(&IVec3::new(chunk_coord.x, chunk_coord.y, chunk_coord.z - 1))
                .map(|chunk| Arc::downgrade(chunk)),
            self.chunk_data_map
                .read()
                .unwrap()
                .get(&IVec3::new(chunk_coord.x, chunk_coord.y, chunk_coord.z + 1))
                .map(|chunk| Arc::downgrade(chunk)),
        ]
    }

    /// Ray cast from the origin until it hits a voxel.
    /// Returns the position of the voxel, the position of the previous voxel and the voxel itself.
    /// If it didn't hit a voxel, returns None.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the ray.
    ///
    /// * `direction` - The direction of the ray.
    ///
    /// * `max_distance` - The maximum distance the ray can travel.
    ///
    /// * `step` - The distance between each step of the ray.
    pub fn ray_casting_voxel(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        step: f32,
    ) -> Option<(IVec3, IVec3, Block)> {
        let mut position = origin;
        let mut last_position = origin;
        let mut last_voxel = None;
        let mut distance = 0.0;

        while distance < max_distance {
            position += direction * step;
            let voxel = self.get_voxel(&World::coord_to_world(position));
            if voxel.is_some() && voxel.unwrap().voxel_type != BlockType::Void {
                last_voxel = voxel;
                break;
            }
            last_position = position;
            distance += step;
        }

        last_voxel.map(|voxel| {
            (
                World::coord_to_world(position),
                World::coord_to_world(last_position),
                voxel,
            )
        })
    }
}

fn setup_world(mut commands: Commands, game_world: Res<GameWorld>) {
    let world = &game_world.world;

    for x in -(WORLD_SIZE - 1)..WORLD_SIZE {
        for z in -(WORLD_SIZE - 1)..WORLD_SIZE {
            let chunk_pos = IVec3::new(x, 0, z);

            world
                .read()
                .unwrap()
                .chunk_entities
                .write()
                .unwrap()
                .insert(chunk_pos, commands.spawn(ChunkEntity(chunk_pos)).id());
        }
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(1.8, 300.0, 1.8).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameWorld>()
            .add_systems(OnEnter(ClientState::LoadingWorld), setup_world);
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
