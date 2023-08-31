pub use crate::prelude::*;

mod core;
mod multiplayer;
mod prelude;
mod terrain;
mod voxel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum ClientState {
    #[default]
    LoadingTexture,
    JoiningServer,
    Playing,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum ServerState {
    #[default]
    LoadingWorld,
    Running,
}
