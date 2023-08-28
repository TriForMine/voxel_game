use crate::voxel::chunk::{ChunkEntity, SIZE};
use crate::voxel::mesh_builder::create_chunk_mesh;
use crate::voxel::world::ChunkDataMap;
use crate::GameWorld;
use crate::ResourcePack;
use bevy::asset::{Assets, Handle};
use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::{
    Added, Commands, Component, Entity, Mesh, Query, Res, ResMut, SystemSet, Transform, Visibility,
    With,
};
use bevy::render::mesh::PrimitiveTopology;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::sync::Arc;

#[derive(Component)]
pub struct ChunkMeshTask(Task<Mesh>);

pub fn prepare_chunks(
    chunks: Query<(Entity, &ChunkEntity), Added<ChunkEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    resource_pack: Res<ResourcePack>,
) {
    for (chunk, chunk_key) in chunks.iter() {
        let mut entity_commands = commands.entity(chunk);
        entity_commands.insert(
            (MaterialMeshBundle {
                material: resource_pack.handle.clone(),
                mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                transform: Transform::from_xyz(
                    (chunk_key.0.x * SIZE) as f32,
                    0.0,
                    (chunk_key.0.z * SIZE) as f32,
                ),
                visibility: Visibility::Hidden,
                ..Default::default()
            }),
        );
    }
}

pub fn clear_dirty_chunks(mut game_world: ResMut<GameWorld>) {
    game_world.world.dirty_chunks.lock().unwrap().clear();
}

pub fn queue_mesh_tasks(mut commands: Commands, game_world: Res<GameWorld>) {
    for chunk_coord in game_world
        .world
        .dirty_chunks
        .lock()
        .unwrap()
        .clone()
        .into_iter()
    {
        let pool = AsyncComputeTaskPool::get();

        let chunk_entities = Arc::clone(&game_world.world.chunk_entities);
        let chunk_entities = chunk_entities.lock().unwrap();
        let chunk_entity = chunk_entities.get(&chunk_coord);

        if let Some(entity) = chunk_entity {
            let chunk_data_map = Arc::clone(&game_world.world.chunk_data_map);

            commands
                .entity(*entity)
                .insert(ChunkMeshTask(pool.spawn(async move {
                    let chunk_data_map = chunk_data_map.lock().unwrap();
                    create_chunk_mesh(&chunk_data_map, &chunk_coord)
                })));
        }
    }
}

pub fn process_mesh_tasks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_query: Query<
        (Entity, &Handle<Mesh>, &mut Visibility, &mut ChunkMeshTask),
        With<ChunkEntity>,
    >,
    mut commands: Commands,
) {
    chunk_query.for_each_mut(|(entity, handle, mut visibility, mut mesh_task)| {
        if let Some(mesh) = future::block_on(future::poll_once(&mut mesh_task.0)) {
            *meshes.get_mut(handle).unwrap() = mesh;
            *visibility = Visibility::Visible;
            commands.entity(entity).remove::<ChunkMeshTask>();
        }
    });
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ChunkMeshingSet;
