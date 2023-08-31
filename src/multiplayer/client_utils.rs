use crate::block::BlockType;
use crate::chunk::Chunk;
use crate::multiplayer::{Channel, ClientMessage, ServerMessage};
use crate::world::GameWorld;
use crate::{connection_config, PendingServerMessage, PROTOCOL_ID};
use bevy::prelude::{Res, ResMut};
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
use bevy_renet::renet::RenetClient;
use std::net::UdpSocket;
use std::time::SystemTime;

pub fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(connection_config());

    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    (client, transport)
}

pub fn client_receive_system(
    mut client: ResMut<RenetClient>,
    mut pending_messages: ResMut<PendingServerMessage>,
) {
    pending_messages.0.clear();

    for channel in [Channel::Reliable, Channel::Chunk] {
        while let Some(message) = client.receive_message(channel) {
            let server_message: ServerMessage = bincode::deserialize(&message).unwrap();

            pending_messages.0.push(server_message);
        }
    }
}

pub fn client_handle_messages(
    mut client: ResMut<RenetClient>,
    transport: Res<NetcodeClientTransport>,
    game_world: Res<GameWorld>,
    mut pending_messages: ResMut<PendingServerMessage>,
) {
    let client_id = transport.client_id();
    for server_message in pending_messages.0.drain(..) {
        match server_message {
            ServerMessage::Ping => {
                println!("Client {} received ping.", client_id);

                let message = bincode::serialize(&ClientMessage::Pong).unwrap();
                client.send_message(Channel::Reliable, message);
            }
            ServerMessage::Pong => {
                println!("Client {} received pong.", client_id);
            }
            ServerMessage::Chunk(chunk_pos, compressed_chunk) => {
                let decompressed_chunk = Chunk::from_compressed(&compressed_chunk);

                game_world
                    .world
                    .write()
                    .unwrap()
                    .set_chunk(chunk_pos, decompressed_chunk);
            }
            ServerMessage::PlayerJoined(id) => {
                println!("Client {} received player joined: {}", client_id, id);
            }
            ServerMessage::PlayerLeft(id) => {
                println!("Client {} received player left: {}", client_id, id);
            }
            ServerMessage::BlockBroken(pos) => {
                game_world
                    .world
                    .write()
                    .unwrap()
                    .edit_voxel(&pos, BlockType::Void);
            }
            ServerMessage::BlockPlaced(pos, block_type) => {
                game_world
                    .world
                    .write()
                    .unwrap()
                    .edit_voxel(&pos, block_type);
            }
        }
    }
}
