pub use crate::prelude::*;

mod core;
mod prelude;
mod terrain;
mod voxel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum ClientState {
    #[default]
    LoadingTexture,
    LoadingWorld,
    Playing,
}
