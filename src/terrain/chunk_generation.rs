use crate::terrain::terrain_generator::TERRAIN_GENERATOR;
use crate::voxel::chunk::{Chunk, ChunkEntity};
use crate::voxel::world::GameWorld;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::sync::{Arc, RwLock};

#[derive(Component)]
pub struct TerrainGenTask(Task<Arc<RwLock<Chunk>>>);

pub fn queue_chunk_generation(
    mut commands: Commands,
    new_chunks: Query<(Entity, &ChunkEntity), Added<ChunkEntity>>,
) {
    new_chunks
        .iter()
        .map(|(entity, key)| (entity, key.0))
        .map(|(entity, chunk_coord)| {
            (
                entity,
                (TerrainGenTask(AsyncComputeTaskPool::get().spawn(async move {
                    let mut chunk: Chunk = Chunk::default();
                    chunk.pos = chunk_coord;
                    TERRAIN_GENERATOR
                        .read()
                        .unwrap()
                        .generate(chunk_coord, &mut chunk.voxels);

                    Arc::new(RwLock::new(chunk))
                }))),
            )
        })
        .for_each(|(entity, gen_task)| {
            commands.entity(entity).insert(gen_task);
        });
}

pub fn process_chunk_generation(
    game_world: Res<GameWorld>,
    mut commands: Commands,
    mut gen_chunks: Query<(Entity, &ChunkEntity, &mut TerrainGenTask)>,
) {
    gen_chunks.for_each_mut(|(entity, chunk_entity, mut gen_task)| {
        if let Some(chunk) = future::block_on(future::poll_once(&mut gen_task.0)) {
            let neighbors = game_world
                .world
                .read()
                .unwrap()
                .get_neighbors_chunks(&chunk_entity.0);

            for i in 0..neighbors.len() {
                let neighbor = neighbors.get(i).unwrap();
                if let Some(ref neighbor) = neighbor {
                    chunk.write().unwrap().set_neighbor(i, neighbor.clone());

                    let neighbor = neighbor.upgrade().unwrap();
                    let mut neighbor = neighbor.write().unwrap();
                    // i ^ 1 is the opposite direction of i (i.e. 0 ^ 1 = 1, 1 ^ 1 = 0, 2 ^ 1 = 3, 3 ^ 1 = 2)
                    neighbor.set_neighbor(i ^ 1, Arc::downgrade(&chunk));

                    game_world
                        .world
                        .read()
                        .unwrap()
                        .dirty_chunks
                        .write()
                        .unwrap()
                        .insert(neighbor.pos);
                }
            }

            game_world
                .world
                .read()
                .unwrap()
                .chunk_data_map
                .write()
                .unwrap()
                .insert(chunk_entity.0, chunk);

            game_world
                .world
                .read()
                .unwrap()
                .dirty_chunks
                .write()
                .unwrap()
                .insert(chunk_entity.0);

            commands.entity(entity).remove::<TerrainGenTask>();
        }
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct TerrainGenSet;
