use crate::block::BlockType;
use crate::chunk::Chunk;
use crate::multiplayer::{Channel, ClientMessage, ServerMessage};
use crate::player::{OtherPlayer, PLAYER_HEIGHT, PLAYER_WIDTH};
use crate::world::GameWorld;
use crate::{
    connection_config, shape, Assets, Color, Commands, Mesh, PbrBundle, PendingServerMessage,
    Query, StandardMaterial, Transform, Vec3, With, PROTOCOL_ID,
};
use bevy::prelude::{Res, ResMut};
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
use bevy_renet::renet::RenetClient;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::SystemTime;

pub fn new_renet_client(server_addr: SocketAddr) -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(connection_config());
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
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

    for channel in [Channel::Reliable, Channel::Unreliable, Channel::Chunk] {
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut player_transforms: Query<&mut Transform, With<OtherPlayer>>,
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
            ServerMessage::PlayerJoined(id, pos) => {
                println!("Client {} received player joined: {}", client_id, id);

                let player_entity = commands.spawn((
                    OtherPlayer { id },
                    PbrBundle {
                        mesh: meshes.add(
                            shape::Capsule {
                                radius: PLAYER_WIDTH,
                                depth: PLAYER_HEIGHT - PLAYER_WIDTH,
                                ..Default::default()
                            }
                            .into(),
                        ),
                        material: materials.add(StandardMaterial {
                            base_color: Color::rgb(0.0, 0.0, 1.0),
                            ..Default::default()
                        }),
                        transform: Transform::from_translation(
                            pos + Vec3::new(0.0, PLAYER_HEIGHT / 2.0, 0.0),
                        )
                        .looking_to(Vec3::Z, Vec3::Y),
                        ..Default::default()
                    },
                ));

                game_world
                    .world
                    .read()
                    .unwrap()
                    .players
                    .write()
                    .unwrap()
                    .insert(id, player_entity.id());
            }
            ServerMessage::PlayerMoved(id, pos) => {
                let world = Arc::clone(&game_world.world);
                let world = world.read().unwrap();
                let players = world.players.read().unwrap();
                let player_entity = players.get(&id);

                if let Some(player_entity) = player_entity {
                    let transform = player_transforms.get_mut(*player_entity);

                    if let Ok(mut transform) = transform {
                        transform.translation = pos + Vec3::new(0.0, PLAYER_HEIGHT / 2.0, 0.0);
                    }
                }
            }
            ServerMessage::PlayerLeft(id) => {
                println!("Client {} received player left: {}", client_id, id);

                if let Some(player_entity) = game_world
                    .world
                    .read()
                    .unwrap()
                    .players
                    .write()
                    .unwrap()
                    .remove(&id)
                {
                    commands.entity(player_entity).despawn();
                }
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
