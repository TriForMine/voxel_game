use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use voxel_game::chunk::ChunkPlugin;
use voxel_game::chunk_generation::TerrainGenSet;
use voxel_game::meshing::ChunkMeshingSet;
use voxel_game::player::{PlayerPlugin, PlayerSet};
use voxel_game::texture::TexturePlugin;
use voxel_game::ui::UIPlugin;
use voxel_game::world::WorldPlugin;
use voxel_game::ClientState;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Voxel Game".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            PlayerPlugin,
            UIPlugin,
            TexturePlugin,
            WorldPlugin,
            ChunkPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .add_state::<ClientState>()
        .configure_set(Update, TerrainGenSet)
        .configure_set(Update, PlayerSet)
        .configure_set(
            Update,
            ChunkMeshingSet.after(TerrainGenSet).after(PlayerSet),
        )
        .run();
}
