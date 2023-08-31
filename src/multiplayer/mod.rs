mod client_utils;
mod message;
mod server_utils;

use crate::{Entity, Resource, SystemSet};
use bevy::prelude::Component;
use bevy_renet::renet::{ChannelConfig, ConnectionConfig, SendType};
use bimap::BiMap;
use std::time::Duration;

pub use client_utils::*;
pub use message::*;
pub use server_utils::*;

pub const PROTOCOL_ID: u64 = 0x1122334455667788;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ReadMessagesSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct HandlingMessagesSet;

#[derive(Debug, Component)]
pub struct NetworkPlayer {
    pub id: u64,
}

#[derive(Debug, Resource, Default)]
pub struct Lobby {
    pub players: BiMap<u64, Entity>,
}

#[derive(Clone, Copy)]
pub enum Channel {
    Reliable,
    ReliableOrdered,
    Unreliable,
    Chunk,
}

impl From<Channel> for u8 {
    fn from(channel: Channel) -> Self {
        match channel {
            Channel::Reliable => 0,
            Channel::ReliableOrdered => 1,
            Channel::Unreliable => 2,
            Channel::Chunk => 3,
        }
    }
}

impl Channel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Channel::Reliable.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableUnordered {
                    resend_time: Duration::from_millis(300),
                },
            },
            ChannelConfig {
                channel_id: Channel::ReliableOrdered.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(300),
                },
            },
            ChannelConfig {
                channel_id: Channel::Unreliable.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Channel::Chunk.into(),
                max_memory_usage_bytes: 100 * 1024 * 1024,
                send_type: SendType::ReliableUnordered {
                    resend_time: Duration::from_millis(300),
                },
            },
        ]
    }
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 5 * 1024 * 1024,
        client_channels_config: Channel::channels_config(),
        server_channels_config: Channel::channels_config(),
    }
}
