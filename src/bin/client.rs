use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::RenetClientPlugin;
use voxel_game::chunk::ClientChunkPlugin;
use voxel_game::chunk_generation::TerrainGenSet;
use voxel_game::meshing::ChunkMeshingSet;
use voxel_game::player::{PlayerPlugin, PlayerSet};
use voxel_game::texture::TexturePlugin;
use voxel_game::ui::UIPlugin;
use voxel_game::world::ClientWorldPlugin;
use voxel_game::{
    client_handle_messages, client_receive_system, new_renet_client, ClientState,
    HandlingMessagesSet, PendingServerMessage, ReadMessagesSet,
};

fn main() {
    let (client, transport) = new_renet_client();

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
            ClientWorldPlugin,
            ClientChunkPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .add_state::<ClientState>()
        .init_resource::<PendingServerMessage>()
        .insert_resource(client)
        .insert_resource(transport)
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
        .run();
}
