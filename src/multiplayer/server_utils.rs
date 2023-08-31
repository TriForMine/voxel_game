use crate::block::BlockType;
use crate::multiplayer::PROTOCOL_ID;
use crate::world::GameWorld;
use crate::{
    connection_config, Channel, ClientMessage, Commands, EventReader, Lobby, NetworkPlayer,
    PendingClientMessage, Res, ResMut, ServerMessage,
};
use bevy_egui::EguiContexts;
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::{RenetServer, ServerEvent};
use renet_visualizer::RenetServerVisualizer;
use std::net::UdpSocket;
use std::time::SystemTime;

pub fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_config = ServerConfig {
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addr,
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(current_time, server_config, socket).unwrap();

    (server, transport)
}

pub fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {} connected.", client_id);
                visualizer.add_client(*client_id);

                let player = commands.spawn(NetworkPlayer { id: *client_id }).id();

                lobby.players.insert(*client_id, player);

                let message = bincode::serialize(&ServerMessage::Ping).unwrap();
                server.send_message(*client_id, Channel::Reliable, message);

                let message = bincode::serialize(&ServerMessage::PlayerJoined(*client_id)).unwrap();
                server.broadcast_message_except(*client_id, Channel::Reliable, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {} disconnected: {}", client_id, reason);
                visualizer.remove_client(*client_id);

                if let Some((_, player)) = lobby.players.remove_by_left(client_id) {
                    commands.entity(player).despawn();
                }

                let message = bincode::serialize(&ServerMessage::PlayerLeft(*client_id)).unwrap();
                server.broadcast_message_except(*client_id, Channel::Reliable, message);
            }
        }
    }
}

pub fn server_receive_system(
    mut server: ResMut<RenetServer>,
    mut pending_messages: ResMut<PendingClientMessage>,
) {
    pending_messages.0.clear();

    for client_id in server.clients_id() {
        for channel in [Channel::Reliable, Channel::Chunk] {
            while let Some(message) = server.receive_message(client_id, channel) {
                let message: ClientMessage = bincode::deserialize(&message).unwrap();

                pending_messages.0.push((client_id, message));
            }
        }
    }
}

pub fn server_handle_messages_system(
    mut pending_messages: ResMut<PendingClientMessage>,
    server_world: Res<GameWorld>,
    mut server: ResMut<RenetServer>,
) {
    for (client_id, message) in pending_messages.0.drain(..) {
        match message {
            ClientMessage::Ping => {}
            ClientMessage::Pong => {}
            ClientMessage::BreakBlock(pos) => {
                server_world
                    .world
                    .write()
                    .unwrap()
                    .edit_voxel(&pos, BlockType::Void);

                let message = bincode::serialize(&ServerMessage::BlockBroken(pos)).unwrap();
                server.broadcast_message(Channel::Reliable, message);
            }
            ClientMessage::PlaceBlock(pos, block_type) => {
                server_world
                    .world
                    .write()
                    .unwrap()
                    .edit_voxel(&pos, block_type);

                let message =
                    bincode::serialize(&ServerMessage::BlockPlaced(pos, block_type)).unwrap();
                server.broadcast_message(Channel::Reliable, message);
            }
            ClientMessage::RequestChunk(coord) => {
                let chunk = server_world.world.read().unwrap().get_chunk(coord);

                if let Some(chunk) = chunk {
                    let chunk = chunk.read().unwrap();

                    let message =
                        bincode::serialize(&ServerMessage::Chunk(coord, chunk.compress())).unwrap();
                    server.send_message(client_id, Channel::Chunk, message);
                } else {
                    println!("Client requested non-existing chunk.");
                }
            }
        }
    }
}

pub fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) {
    visualizer.update(&server);
    visualizer.show_window(egui_contexts.ctx_mut());
}
