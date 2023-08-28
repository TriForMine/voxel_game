use crate::terrain::terrain_generator::TERRAIN_GENERATOR;
use crate::voxel::chunk::{Chunk, ChunkData, ChunkEntity, HEIGHT, SIZE};
use crate::voxel::voxel::Voxel;
use crate::GameWorld;
use bevy::prelude::{Added, Commands, Component, Entity, Query, ResMut, SystemSet};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

#[derive(Component)]
pub struct TerrainGenTask(Task<ChunkData>);

pub fn queue_chunk_generation(
    mut commands: Commands,
    new_chunks: Query<(Entity, &ChunkEntity), Added<ChunkEntity>>,
) {
    let parallel_tasks = AsyncComputeTaskPool::get().thread_num();

    new_chunks
        .iter()
        .map(|(entity, key)| (entity, key.0))
        .map(|(entity, chunk_coord)| {
            (
                entity,
                (TerrainGenTask(AsyncComputeTaskPool::get().spawn(async move {
                    let mut chunk_data: ChunkData =
                        [Voxel::new_empty(); (SIZE * SIZE * HEIGHT) as usize];
                    TERRAIN_GENERATOR
                        .read()
                        .unwrap()
                        .generate(chunk_coord, &mut chunk_data);
                    chunk_data
                }))),
            )
        })
        .for_each(|(entity, gen_task)| {
            commands.entity(entity).insert(gen_task);
        });
}

pub fn process_chunk_generation(
    mut game_world: ResMut<GameWorld>,
    mut commands: Commands,
    mut gen_chunks: Query<(Entity, &ChunkEntity, &mut TerrainGenTask)>,
) {
    gen_chunks.for_each_mut(|(entity, chunk, mut gen_task)| {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut gen_task.0)) {
            game_world
                .world
                .chunk_data_map
                .lock()
                .unwrap()
                .insert(chunk.0, Chunk { voxels: chunk_data });

            game_world
                .world
                .dirty_chunks
                .lock()
                .unwrap()
                .insert(chunk.0);

            commands.entity(entity).remove::<TerrainGenTask>();
        }
    })
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct TerrainGenSet;
