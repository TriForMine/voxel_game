pub use crate::prelude::*;

mod core;
mod discord_presence;
mod multiplayer;
mod prelude;
mod terrain;
mod voxel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum ClientState {
    #[default]
    LoadingTexture,
    MainMenu,
    JoiningServer,
    LoadingWorld,
    Playing,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum ClientMode {
    #[default]
    SinglePlayer,
    Lan,
    Online,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum ServerState {
    #[default]
    MainMenu,
    LoadingWorld,
    Running,
}
