use crate::block::BlockType;
use crate::chunk::CompressedChunk;
use crate::IVec3;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Resource)]
pub struct PendingClientMessage(pub(crate) Vec<(u64, ClientMessage)>);

#[derive(Debug, Default, Resource)]
pub struct PendingServerMessage(pub(crate) Vec<ServerMessage>);

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Ping,
    Pong,
    BreakBlock(IVec3),
    PlaceBlock(IVec3, BlockType),
    RequestChunk(IVec3),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ping,
    Pong,
    Chunk(IVec3, CompressedChunk),
    PlayerJoined(u64),
    PlayerLeft(u64),
    BlockBroken(IVec3),
    BlockPlaced(IVec3, BlockType),
}
