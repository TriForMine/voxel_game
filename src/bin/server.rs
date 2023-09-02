use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_renet::transport::NetcodeServerPlugin;
use bevy_renet::RenetServerPlugin;
use renet_visualizer::RenetServerVisualizer;
use voxel_game::chunk::ServerChunkPlugin;
use voxel_game::world::{GameWorld, ServerWorldPlugin};
use voxel_game::{
    new_renet_server, server_handle_messages_system, server_receive_system, server_update_system,
    update_visualizer_system, Lobby, PendingClientMessage, ReadMessagesSet, ServerState,
};

fn main() {
    let (server, transport) = new_renet_server(64);

    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Game Server".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),))
        .insert_resource(server)
        .insert_resource(transport)
        .add_state::<ServerState>()
        .add_plugins((
            RenetServerPlugin,
            NetcodeServerPlugin,
            EguiPlugin,
            FrameTimeDiagnosticsPlugin,
            ServerWorldPlugin,
            ServerChunkPlugin,
        ))
        .init_resource::<Lobby>()
        .init_resource::<GameWorld>()
        .init_resource::<PendingClientMessage>()
        .insert_resource(RenetServerVisualizer::<200>::default())
        .add_systems(Startup, force_server_state_to_running_system)
        .configure_set(PreUpdate, ReadMessagesSet)
        .add_systems(PreUpdate, server_receive_system.in_set(ReadMessagesSet))
        .add_systems(
            Update,
            (
                update_visualizer_system,
                server_update_system,
                server_handle_messages_system,
            )
                .run_if(in_state(ServerState::Running)),
        )
        .run();
}

fn force_server_state_to_running_system(mut next_state: ResMut<NextState<ServerState>>) {
    next_state.set(ServerState::LoadingWorld);
}
