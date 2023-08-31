use crate::chunk::ServerChunkEntity;
use crate::terrain::chunk_generation::TerrainGenTask;
use crate::voxel::chunk::{ChunkEntity, SIZE};
use crate::voxel::mesh_builder::create_chunk_mesh;
use crate::voxel::texture::ResourcePack;
use crate::voxel::world::GameWorld;
use crate::{ClientState, ServerState};
use bevy::asset::{Assets, Handle};
use bevy::pbr::MaterialMeshBundle;
use bevy::prelude::*;
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
        entity_commands.insert(MaterialMeshBundle {
            material: resource_pack.handle.clone(),
            mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
            transform: Transform::from_xyz(
                (chunk_key.0.x * SIZE) as f32,
                0.0,
                (chunk_key.0.z * SIZE) as f32,
            ),
            visibility: Visibility::Hidden,
            ..Default::default()
        });
    }
}

pub fn clear_dirty_chunks(game_world: Res<GameWorld>) {
    game_world
        .world
        .write()
        .unwrap()
        .dirty_chunks
        .write()
        .unwrap()
        .clear();
}

pub fn queue_mesh_tasks(mut commands: Commands, game_world: Res<GameWorld>) {
    for chunk_coord in game_world
        .world
        .read()
        .unwrap()
        .dirty_chunks
        .read()
        .unwrap()
        .clone()
        .into_iter()
    {
        let pool = AsyncComputeTaskPool::get();

        let chunk_entities = Arc::clone(&game_world.world.read().unwrap().chunk_entities);
        let chunk_entities = chunk_entities.read().unwrap();
        let chunk_entity = chunk_entities.get(&chunk_coord);

        if let Some(entity) = chunk_entity {
            let chunk_data_map = Arc::clone(&game_world.world.read().unwrap().chunk_data_map);

            commands
                .entity(*entity)
                .insert(ChunkMeshTask(pool.spawn(async move {
                    let chunk_data_map = chunk_data_map.read().unwrap();
                    let chunk = chunk_data_map.get(&chunk_coord).unwrap().read().unwrap();
                    create_chunk_mesh(&chunk)
                })));
        } else {
            println!("Chunk {:?} not found", chunk_coord);
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

pub fn check_server_loading_world_ended(
    gen_tasks: Query<(Entity, &ServerChunkEntity, &mut TerrainGenTask)>,
    mut next_state: ResMut<NextState<ServerState>>,
) {
    if gen_tasks.is_empty() {
        println!("Server is ready!");
        next_state.set(ServerState::Running);
    }
}

pub fn check_loading_world_ended(
    client_world: Res<GameWorld>,
    mut next_state: ResMut<NextState<ClientState>>,
) {
    if client_world
        .world
        .read()
        .unwrap()
        .pending_requested_chunks
        .read()
        .unwrap()
        .len()
        == 0
    {
        next_state.set(ClientState::Playing);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ChunkMeshingSet;
