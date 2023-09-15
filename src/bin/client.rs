#![windows_subsystem = "windows"]

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_discord_presence::{ActivityState, RPCConfig, RPCPlugin};
use discord_presence::models::{ActivityAssets};
use bevy_renet::transport::{NetcodeClientPlugin, NetcodeServerPlugin};
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use voxel_game::chunk::{ClientChunkPlugin, ServerChunkPlugin};
use voxel_game::chunk_generation::TerrainGenSet;
use voxel_game::meshing::ChunkMeshingSet;
use voxel_game::player::{PlayerPlugin, PlayerSet};
use voxel_game::texture::TexturePlugin;
use voxel_game::ui::{MainMenuState, UIPlugin};
use voxel_game::world::{ClientWorldPlugin, ServerWorldPlugin};
use voxel_game::{client_handle_messages, client_receive_system, server_handle_messages_system, server_receive_system, server_update_system, ClientState, HandlingMessagesSet, Lobby, PendingClientMessage, PendingServerMessage, ReadMessagesSet, ServerState, ClientMode};

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voxel Game".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
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
            }
        ))
        .add_state::<ClientState>()
        .add_state::<ClientMode>()
        .add_state::<ServerState>()
        .add_state::<MainMenuState>()
        .init_resource::<Lobby>()
        .init_resource::<PendingClientMessage>()
        .init_resource::<PendingServerMessage>()
        .configure_set(PreUpdate, ReadMessagesSet)
        .configure_set(Update, HandlingMessagesSet)
        .configure_set(Update, PlayerSet.after(HandlingMessagesSet))
        .configure_set(
            Update,
            ChunkMeshingSet
                .after(TerrainGenSet)
                .after(PlayerSet)
                .after(HandlingMessagesSet),
        )
        .add_systems(
            PreUpdate,
            client_receive_system
                .run_if(bevy_renet::transport::client_connected())
                .in_set(ReadMessagesSet),
        )
        .add_systems(
            Update,
            client_handle_messages
                .run_if(bevy_renet::transport::client_connected())
                .in_set(HandlingMessagesSet),
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
        .add_systems(
            PreUpdate,
            update_presence
        )
        .run();
}

fn update_presence(
    mut state: ResMut<ActivityState>,
    app_state: Res<State<ClientState>>,
    client_mode: Res<State<ClientMode>>,
    mut last_state: Local<Option<ClientState>>,
) {
    if *last_state != Some(*app_state.get()) {
        println!("State changed to {:?}", app_state.get());

        state.assets = Some(ActivityAssets {
            large_image: Some("logo".to_string()),
            large_text: Some("Voxel Game".to_string()),
            small_image: Some("triformine".to_string()),
            small_text: Some("Ceated by @TriForMine".to_string())
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
