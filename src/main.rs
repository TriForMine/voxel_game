#![feature(let_chains)]

mod core;
mod terrain;
mod voxel;

use crate::core::player::{Player, PlayerCamera, PlayerPlugin, PlayerSet};
use crate::terrain::meshing::{
    check_loading_world_ended, clear_dirty_chunks, prepare_chunks, process_mesh_tasks,
    queue_mesh_tasks, ChunkMeshingSet,
};
use crate::terrain::terrain::{process_chunk_generation, queue_chunk_generation, TerrainGenSet};
use crate::voxel::chunk::ChunkEntity;
use crate::voxel::world::World;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AddressMode, FilterMode, SamplerDescriptor};
use bevy::render::texture::ImageSampler;
use bevy_egui::egui::RichText;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::sync::{Arc, RwLock};

pub const WORLD_SIZE: i32 = 5;

#[derive(Resource)]
pub struct ResourcePack {
    handle: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct GameWorld {
    world: Arc<RwLock<World>>,
}

impl Default for GameWorld {
    fn default() -> Self {
        Self {
            world: Arc::new(RwLock::new(World::new())),
        }
    }
}

#[derive(Resource, Default)]
struct TexturePackLoading(Handle<Image>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    LoadingTexture,
    LoadingWorld,
    Playing,
}

fn main() {
    App::new()
        .init_resource::<GameWorld>()
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
        .configure_set(Update, PlayerSet)
        .configure_set(
            Update,
            ChunkMeshingSet.after(TerrainGenSet).after(PlayerSet),
        )
        .add_systems(Startup, setup_texture)
        .add_systems(OnEnter(AppState::LoadingWorld), setup_world)
        .add_systems(
            Update,
            debug_menu_system.run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            loading_menu_system.run_if(
                in_state(AppState::LoadingTexture).or_else(in_state(AppState::LoadingWorld)),
            ),
        )
        .add_systems(
            Last,
            check_loading_world_ended.run_if(in_state(AppState::LoadingWorld)),
        )
        .add_systems(
            Update,
            check_assets_ready.run_if(in_state(AppState::LoadingTexture)),
        )
        .add_systems(
            Update,
            (queue_chunk_generation, process_chunk_generation)
                .chain()
                .in_set(TerrainGenSet)
                .run_if(in_state(AppState::LoadingWorld).or_else(in_state(AppState::Playing))),
        )
        .add_systems(
            Update,
            (prepare_chunks, queue_mesh_tasks, process_mesh_tasks)
                .chain()
                .in_set(ChunkMeshingSet)
                .run_if(in_state(AppState::LoadingWorld).or_else(in_state(AppState::Playing))),
        )
        .add_systems(
            Last,
            clear_dirty_chunks
                .run_if(in_state(AppState::LoadingWorld).or_else(in_state(AppState::Playing))),
        )
        .run();
}

fn loading_menu_system(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading("Loading");
    });
}

fn debug_menu_system(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<(&Player, &Transform), (With<Player>, Without<PlayerCamera>)>,
) {
    if let Ok((player, player_transform)) = player_query.get_single() {
        let fps = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.average());

        let player_pos = World::coord_to_world(player_transform.translation);
        let mut chunk_pos = IVec3::new(0, 0, 0);
        let mut local_pos = player_pos;
        World::make_coords_valid(&mut chunk_pos, &mut local_pos);

        // Make a invisible window in center of the screen to display some sort of cursor or crosshair
        egui::Window::new("Cursor")
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::none())
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(
                    RichText::new("X")
                        .color(egui::Color32::from_rgb(255, 255, 255))
                        .heading(),
                );
            });

        egui::Window::new("Debug")
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::none())
            .title_bar(false)
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(10.0, 10.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!("FPS: {:?}", fps.unwrap_or_default().round()),
                );

                ui.separator();

                ui.heading(
                    RichText::new("Position")
                        .color(egui::Color32::from_rgb(255, 255, 255))
                        .heading(),
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!(
                        "World Position: X: {:?} Y: {:?} Z: {:?}",
                        player_pos.x, player_pos.y, player_pos.z
                    ),
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!("Chunk Position: X: {:?} Z: {:?}", chunk_pos.x, chunk_pos.z),
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!(
                        "Local Position: X: {:?} Y: {:?} Z: {:?}",
                        local_pos.x, local_pos.y, local_pos.z
                    ),
                );

                if let Some(looking_at_pos) = player.looking_at_pos {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 255, 255),
                        format!(
                            "Looking At: X: {:?} Y: {:?} Z: {:?}",
                            looking_at_pos.x, looking_at_pos.y, looking_at_pos.z
                        ),
                    );
                }
            });
    }
}

fn setup_texture(
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
            next_state.set(AppState::LoadingWorld);
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}
