#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use bevy::color::palettes::css::WHITE;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy_renet::netcode::{NetcodeClientPlugin, NetcodeServerPlugin};
use bevy_renet::{client_connected, RenetClientPlugin, RenetServerPlugin};
use discord_presence::models::rich_presence::ActivityAssets;
use voxel_game::chunk::{ClientChunkPlugin, ServerChunkPlugin};
use voxel_game::chunk_generation::TerrainGenSet;
use voxel_game::meshing::ChunkMeshingSet;
use voxel_game::player::{PlayerPlugin, PlayerSet};
use voxel_game::texture::TexturePlugin;
use voxel_game::ui::{MainMenuState, UIPlugin};
use voxel_game::world::{ClientWorldPlugin, ServerWorldPlugin};
use voxel_game::{
    client_handle_messages, client_receive_system, server_handle_messages_system,
    server_receive_system, server_update_system, ActivityState, ClientMode, ClientState,
    HandlingMessagesSet, Lobby, PendingClientMessage, PendingServerMessage, RPCConfig, RPCPlugin,
    ReadMessagesSet, ServerState,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Voxel Game".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
            WireframePlugin,
            PlayerPlugin,
            UIPlugin,
            TexturePlugin,
            RenetClientPlugin,
            NetcodeClientPlugin,
            ClientWorldPlugin,
            ClientChunkPlugin,
            FrameTimeDiagnosticsPlugin,
            RenetServerPlugin,
            NetcodeServerPlugin,
            ServerWorldPlugin,
            ServerChunkPlugin,
            RPCPlugin {
                config: RPCConfig {
                    app_id: 1147947143458472026,
                    show_time: true,
                },
            },
        ))
        .init_state::<ClientState>()
        .init_state::<ClientMode>()
        .init_state::<ServerState>()
        .init_state::<MainMenuState>()
        .init_resource::<Lobby>()
        .init_resource::<PendingClientMessage>()
        .init_resource::<PendingServerMessage>()
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into()
        })
        .configure_sets(PreUpdate, ReadMessagesSet)
        .configure_sets(Update, HandlingMessagesSet)
        .configure_sets(Update, PlayerSet.after(HandlingMessagesSet))
        .configure_sets(
            Update,
            ChunkMeshingSet
                .after(TerrainGenSet)
                .after(PlayerSet)
                .after(HandlingMessagesSet),
        )
        .add_systems(
            PreUpdate,
            client_receive_system
                .in_set(ReadMessagesSet)
                .run_if(client_connected),
        )
        .add_systems(
            Update,
            client_handle_messages
                .in_set(HandlingMessagesSet)
                .run_if(client_connected),
        )
        .add_systems(
            PreUpdate,
            server_receive_system
                .in_set(ReadMessagesSet)
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(
            Update,
            (server_update_system, server_handle_messages_system)
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(PreUpdate, update_presence)
        .run();
}

fn update_presence(
    mut state: ResMut<ActivityState>,
    app_state: Res<State<ClientState>>,
    client_mode: Res<State<ClientMode>>,
    mut last_state: Local<Option<ClientState>>,
) {
    if *last_state != Some(*app_state.get()) {
        state.assets = Some(ActivityAssets {
            large_image: Some("logo".to_string()),
            large_text: Some("Voxel Game".to_string()),
            small_image: Some("triformine".to_string()),
            small_text: Some("Ceated by @TriForMine".to_string()),
        });

        match app_state.get() {
            ClientState::MainMenu => {
                state.instance = Some(false);
                state.details = Some("Main Menu".to_string());
                state.state = Some("In the main menu".to_string());
            }
            ClientState::JoiningServer => {
                state.instance = Some(false);
                state.details = Some("Joining Server".to_string());
                state.state = None;
            }
            ClientState::LoadingWorld => {
                state.instance = Some(false);
                state.details = Some("Loading World".to_string());
                state.state = None;
            }
            ClientState::Playing => {
                state.instance = Some(true);
                state.details = Some("Playing".to_string());

                match client_mode.get() {
                    ClientMode::SinglePlayer => {
                        state.state = Some("Singleplayer".to_string());
                    }
                    ClientMode::Lan => {
                        state.state = Some("LAN".to_string());
                    }
                    ClientMode::Online => {
                        state.state = Some("Online".to_string());
                    }
                }
            }
            _ => {}
        }

        *last_state = Some(*app_state.get());
    }
}
