#![feature(let_chains)]

mod flycam;
mod terrain;
mod voxel;

use crate::flycam::CameraTag;
use crate::flycam::PlayerPlugin;
use crate::terrain::meshing::{
    clear_dirty_chunks, prepare_chunks, process_mesh_tasks, queue_mesh_tasks, ChunkMeshingSet,
};
use crate::terrain::terrain::{process_chunk_generation, queue_chunk_generation, TerrainGenSet};
use crate::voxel::chunk::ChunkEntity;
use crate::voxel::world::World;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::bundle::DynamicBundle;
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode, SamplerDescriptor};
use bevy::render::texture::ImageSampler;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

pub const WORLD_SIZE: i32 = 5;

#[derive(Resource)]
pub struct ResourcePack {
    handle: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct GameWorld {
    world: World,
}

#[derive(Resource, Default)]
struct TexturePackLoading(Handle<Image>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Loading,
    Playing,
}

fn main() {
    App::new()
        .init_resource::<TexturePackLoading>()
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins,
            PlayerPlugin,
            FrameTimeDiagnosticsPlugin,
            EguiPlugin,
        ))
        .add_state::<AppState>()
        .configure_set(Update, TerrainGenSet)
        .configure_set(Update, ChunkMeshingSet.after(TerrainGenSet))
        .add_systems(Startup, setup)
        .add_systems(Update, debug_menu_system)
        .add_systems(
            Update,
            check_assets_ready.run_if(in_state(AppState::Loading)),
        )
        .add_systems(
            Update,
            (queue_chunk_generation, process_chunk_generation)
                .chain()
                .in_set(TerrainGenSet),
        )
        .add_systems(
            Update,
            (prepare_chunks, queue_mesh_tasks, process_mesh_tasks)
                .chain()
                .in_set(ChunkMeshingSet)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Last, clear_dirty_chunks.run_if(in_state(AppState::Playing)))
        .run();
}

fn debug_menu_system(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    camera_query: Query<&Transform, With<CameraTag>>,
) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average());

    let camera_pos = camera_query.single().translation.as_ivec3();
    let mut chunk_pos = IVec3::new(0, 0, 0);
    let mut local_pos = camera_pos;
    World::make_coords_valid(&mut chunk_pos, &mut local_pos);

    egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("FPS: {:?}", fps.unwrap_or_default().round()));

        ui.separator();

        ui.heading("Position");
        ui.label(format!(
            "World Position: X: {:?} Y: {:?} Z: {:?}",
            camera_pos.x, camera_pos.y, camera_pos.z
        ));
        ui.label(format!(
            "Chunk Position: X: {:?} Z: {:?}",
            chunk_pos.x, chunk_pos.z
        ));
        ui.label(format!(
            "Local Position: X: {:?} Y: {:?} Z: {:?}",
            local_pos.x, local_pos.y, local_pos.z
        ));
    });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut loading: ResMut<TexturePackLoading>,
) {
    let custom_texture_handle: Handle<Image> = asset_server.load("textures/spritesheet_blocks.png");

    *loading = TexturePackLoading(custom_texture_handle.clone());

    let resource_pack = materials.add(StandardMaterial {
        base_color_texture: Some(custom_texture_handle),
        unlit: true,
        ..Default::default()
    });

    commands.insert_resource(ResourcePack {
        handle: resource_pack,
    });

    let mut world = World::new();

    for x in -(WORLD_SIZE - 1)..WORLD_SIZE {
        for z in -(WORLD_SIZE - 1)..WORLD_SIZE {
            let chunk_pos = IVec3::new(x, 0, z);

            world
                .chunk_entities
                .lock()
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

    commands.insert_resource(GameWorld { world });
}

fn check_assets_ready(
    mut commands: Commands,
    server: Res<AssetServer>,
    loading: Res<TexturePackLoading>,
    mut next_state: ResMut<NextState<AppState>>,
    mut images: ResMut<Assets<Image>>,
) {
    use bevy::asset::LoadState;

    match server.get_load_state(loading.0.clone()) {
        LoadState::Loaded => {
            let image = images.get_mut(&loading.0).unwrap();
            image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                address_mode_u: AddressMode::ClampToBorder,
                address_mode_v: AddressMode::ClampToBorder,
                address_mode_w: AddressMode::ClampToBorder,
                ..default()
            });

            commands.remove_resource::<TexturePackLoading>();
            next_state.set(AppState::Playing);
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}
