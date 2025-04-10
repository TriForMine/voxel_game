use crate::block::BlockType;
use crate::chunk::ServerChunkEntity;
use crate::multiplayer::PROTOCOL_ID;
use crate::quad::HALF_SIZE;
use crate::world::GameWorld;
use crate::{
    connection_config, Channel, ClientMessage, Commands, EventReader, IVec2, Lobby, NetworkPlayer,
    PendingClientMessage, Query, Res, ResMut, ServerMessage, Transform, Vec3,
};
use bevy_egui::EguiContexts;
use bevy_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::{RenetServer, ServerEvent};
use bincode::config;
use local_ip_address::local_ip;
use renet_visualizer::RenetServerVisualizer;
use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::SystemTime;

pub fn new_renet_server(singleplayer: bool) -> (RenetServer, NetcodeServerTransport, SocketAddr) {
    let server = RenetServer::new(connection_config());

    let mut public_addr = SocketAddr::new(local_ip().unwrap(), if singleplayer { 0 } else { 5000 });

    let socket = loop {
        match UdpSocket::bind(public_addr) {
            Ok(socket) => break socket,
            Err(_) => {
                let port = public_addr.port();
                let ip = public_addr.ip();
                println!("Address {}:{} already in use.", ip, port);
                public_addr = SocketAddr::new(ip, port + 1);
            }
        }
    };

    public_addr.set_port(socket.local_addr().unwrap().port());

    println!("Server started on {}", public_addr);

    let current_time: std::time::Duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let server_config = ServerConfig {
        current_time,
        max_clients: if singleplayer { 1 } else { 64 },
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    (server, transport, public_addr)
}

pub fn server_update_system(
    mut server_events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    mut lobby: ResMut<Lobby>,
    mut commands: Commands,
    game_world: Res<GameWorld>,
    players: Query<&NetworkPlayer>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let highest_block = game_world
                    .world
                    .read()
                    .unwrap()
                    .get_highest_block_at_coord(&IVec2::new(0, 0))
                    .as_vec3();

                println!("Client {} connected.", client_id);
                visualizer.add_client(*client_id);

                let position = Vec3::new(
                    highest_block.x,
                    highest_block.y + HALF_SIZE + 2.,
                    highest_block.z,
                );

                let player = commands
                    .spawn(NetworkPlayer {
                        id: *client_id,
                        transform: Transform::from_translation(position)
                            .looking_to(Vec3::Z, Vec3::Y),
                    })
                    .id();

                lobby.players.insert(*client_id, player);

                let message =
                    bincode::serde::encode_to_vec(&ServerMessage::Ping, config::standard())
                        .unwrap();
                server.send_message(*client_id, Channel::Reliable, message);

                // Send all players to the new player
                for (id, player) in lobby.players.iter() {
                    if *id == *client_id {
                        continue;
                    }
                    let position = players.get(*player).unwrap().transform.translation;
                    let message = bincode::serde::encode_to_vec(
                        ServerMessage::PlayerJoined(*id, position),
                        config::standard(),
                    )
                    .unwrap();
                    server.send_message(*client_id, Channel::Reliable, message);
                }

                let message = bincode::serde::encode_to_vec(
                    ServerMessage::PlayerJoined(*client_id, position),
                    config::standard(),
                )
                .unwrap();
                server.broadcast_message_except(*client_id, Channel::Reliable, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {} disconnected: {}", client_id, reason);
                visualizer.remove_client(*client_id);

                if let Some((_, player)) = lobby.players.remove_by_left(client_id) {
                    commands.entity(player).despawn();
                }

                let message = bincode::serde::encode_to_vec(
                    ServerMessage::PlayerLeft(*client_id),
                    config::standard(),
                )
                .unwrap();
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
        for channel in [Channel::Reliable, Channel::Unreliable, Channel::Chunk] {
            while let Some(message) = server.receive_message(client_id, channel) {
                let message: ClientMessage =
                    bincode::serde::decode_from_slice(&message, config::standard())
                        .unwrap()
                        .0;

                pending_messages.0.push((client_id, message));
            }
        }
    }
}

pub fn server_handle_messages_system(
    mut pending_messages: ResMut<PendingClientMessage>,
    server_world: Res<GameWorld>,
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
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

                let message = bincode::serde::encode_to_vec(
                    ServerMessage::BlockBroken(pos),
                    config::standard(),
                )
                .unwrap();
                server.broadcast_message(Channel::Reliable, message);
            }
            ClientMessage::PlayerMoved(pos) => {
                let message = bincode::serde::encode_to_vec(
                    ServerMessage::PlayerMoved(client_id, pos),
                    config::standard(),
                )
                .unwrap();
                server.broadcast_message_except(client_id, Channel::Unreliable, message);
            }
            ClientMessage::PlaceBlock(pos, block_type) => {
                server_world
                    .world
                    .write()
                    .unwrap()
                    .edit_voxel(&pos, block_type);

                let message = bincode::serde::encode_to_vec(
                    ServerMessage::BlockPlaced(pos, block_type),
                    config::standard(),
                )
                .unwrap();
                server.broadcast_message(Channel::Reliable, message);
            }
            ClientMessage::RequestChunk(coord) => {
                let chunk = server_world.world.read().unwrap().get_chunk(coord);

                if let Some(chunk) = chunk {
                    let chunk = chunk.read().unwrap();

                    let message = bincode::serde::encode_to_vec(
                        ServerMessage::Chunk(coord, chunk.compress()),
                        config::standard(),
                    )
                    .unwrap();
                    server.send_message(client_id, Channel::Chunk, message);
                } else {
                    let world = Arc::clone(&server_world.world);
                    let world = world.read().unwrap();

                    let mut pending_generating_chunks =
                        world.pending_generating_chunks.write().unwrap();
                    let pending_generating_chunk = pending_generating_chunks.get_mut(&coord);

                    if pending_generating_chunk.is_none() {
                        let mut pending_generating_chunk = HashSet::new();
                        pending_generating_chunk.insert(client_id);

                        pending_generating_chunks.insert(coord, pending_generating_chunk);
                    } else if let Some(pending_generating_chunk) = pending_generating_chunk {
                        pending_generating_chunk.insert(client_id);
                    }

                    world
                        .chunk_entities
                        .write()
                        .unwrap()
                        .insert(coord, commands.spawn(ServerChunkEntity(coord)).id());
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
