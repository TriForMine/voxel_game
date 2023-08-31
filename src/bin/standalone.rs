use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_renet::transport::{NetcodeClientPlugin, NetcodeServerPlugin};
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use renet_visualizer::RenetServerVisualizer;
use voxel_game::chunk::{ClientChunkPlugin, ServerChunkPlugin};
use voxel_game::chunk_generation::TerrainGenSet;
use voxel_game::meshing::ChunkMeshingSet;
use voxel_game::player::{PlayerPlugin, PlayerSet};
use voxel_game::texture::TexturePlugin;
use voxel_game::ui::UIPlugin;
use voxel_game::world::{ClientWorldPlugin, ServerWorldPlugin};
use voxel_game::{
    client_handle_messages, client_receive_system, new_renet_client, new_renet_server,
    server_handle_messages_system, server_receive_system, server_update_system, ClientState,
    HandlingMessagesSet, Lobby, PendingClientMessage, PendingServerMessage, ReadMessagesSet,
    ServerState,
};

fn main() {
    let (client, client_transport) = new_renet_client();
    let (server, server_transport) = new_renet_server();

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
            RenetServerPlugin,
            NetcodeServerPlugin,
            ServerWorldPlugin,
            ServerChunkPlugin,
            PlayerPlugin,
            UIPlugin,
            TexturePlugin,
            ClientWorldPlugin,
            ClientChunkPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .add_state::<ClientState>()
        .add_state::<ServerState>()
        .init_resource::<Lobby>()
        .init_resource::<PendingServerMessage>()
        .init_resource::<PendingClientMessage>()
        .insert_resource(client)
        .insert_resource(client_transport)
        .insert_resource(server)
        .insert_resource(server_transport)
        .insert_resource(RenetServerVisualizer::<200>::default())
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
        .add_plugins((RenetClientPlugin, NetcodeClientPlugin))
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
        .add_systems(PreUpdate, server_receive_system.in_set(ReadMessagesSet))
        .add_systems(
            Update,
            (server_update_system, server_handle_messages_system)
                .run_if(in_state(ServerState::Running)),
        )
        .run();
}
