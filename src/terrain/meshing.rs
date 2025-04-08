use crate::chunk::ServerChunkEntity;
use crate::terrain::chunk_generation::TerrainGenTask;
use crate::voxel::chunk::{ChunkEntity, SIZE};
use crate::voxel::mesh_builder::create_chunk_mesh;
use crate::voxel::texture::ResourcePack;
use crate::voxel::world::GameWorld;
use crate::{ClientState, ServerState};
use bevy::asset::Assets;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::primitives::Aabb;
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
        entity_commands.insert((
            Transform::from_xyz(
                (chunk_key.0.x * SIZE) as f32,
                0.0,
                (chunk_key.0.z * SIZE) as f32,
            ),
            Visibility::Hidden,
        ));
        debug!("Prepared chunk entity placeholder for {:?}", chunk_key.0);
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
    mut task_query: Query<
        (Entity, &ChunkEntity, &mut Visibility, &mut ChunkMeshTask),
        With<ChunkEntity>,
    >,
    // Query only Mesh3d and the Material
    mut mesh_query: Query<(
        Option<&mut Mesh3d>,
        Option<&mut MeshMaterial3d<StandardMaterial>>, // Still need the material
    )>,
    mut commands: Commands,
    resource_pack: Res<ResourcePack>,
) {
    for (entity, chunk_key, mut visibility, mut mesh_task) in task_query.iter_mut() {
        if let Some(new_mesh) = future::block_on(future::poll_once(&mut mesh_task.0)) {
            let vertex_count = new_mesh.count_vertices();
            let index_count = new_mesh.indices().map_or(0, |indices| indices.len());

            debug!(
                "Processing mesh task for chunk {:?}: Vertices={}, Indices={}",
                chunk_key.0, vertex_count, index_count
            );

            if vertex_count == 0 || index_count == 0 {
                warn!(
                    "Generated mesh for chunk {:?} is empty. Setting visibility to hidden.",
                    chunk_key.0
                );
                // Only remove Mesh3d and the material if they existed
                if mesh_query.get(entity).is_ok() {
                    commands
                        .entity(entity)
                        .remove::<(Mesh3d, MeshMaterial3d<StandardMaterial>)>();
                }
                *visibility = Visibility::Hidden;
            } else {
                let new_mesh_handle = meshes.add(new_mesh);

                // Try to get existing components mutably
                if let Ok((mut maybe_mesh_3d, mut maybe_material)) = mesh_query.get_mut(entity) {
                    // Entity already has components (or some of them)
                    if let Some(mut mesh_3d) = maybe_mesh_3d {
                        // Update existing mesh handle
                        debug!("Updating existing mesh for chunk {:?}", chunk_key.0);
                        if mesh_3d.0 != new_mesh_handle {
                            // Only update if handle actually changed
                            mesh_3d.0 = new_mesh_handle.clone();
                        }
                    } else {
                        // Mesh component missing, insert it
                        debug!("Inserting Mesh3d for chunk {:?}", chunk_key.0);
                        commands
                            .entity(entity)
                            .insert(Mesh3d(new_mesh_handle.clone()));
                    }

                    // Check and insert material if missing
                    if maybe_material.is_none() {
                        debug!("Inserting MeshMaterial3d for chunk {:?}", chunk_key.0);
                        commands
                            .entity(entity)
                            .insert(MeshMaterial3d(resource_pack.handle.clone()));
                    }
                } else {
                    // Entity likely had no mesh components before, insert both
                    debug!("Attaching new mesh and material to chunk {:?}", chunk_key.0);
                    commands.entity(entity).insert((
                        MeshMaterial3d(resource_pack.handle.clone()),
                        Mesh3d(new_mesh_handle),
                    ));
                }
                // Make it visible
                *visibility = Visibility::Visible;
            }

            // Remove the task component once processed
            commands.entity(entity).remove::<ChunkMeshTask>();
        }
    }
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
