use crate::core::player::{Player, PlayerCamera};
use crate::voxel::world::World;
use crate::ClientState;
use bevy::app::App;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::math::IVec3;
use bevy::prelude::*;
use bevy_egui::egui::RichText;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn loading_menu_system(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading("Loading");
    });
}

fn debug_menu_system(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<(&Player, &Transform), (With<Player>, Without<PlayerCamera>)>,
) {
    if let Ok((player, player_transform)) = player_query.get_single() {
        let fps = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.average());

        let player_pos = World::coord_to_world(player_transform.translation);
        let mut chunk_pos = IVec3::new(0, 0, 0);
        let mut local_pos = player_pos;
        World::make_coords_valid(&mut chunk_pos, &mut local_pos);

        // Make a invisible window in center of the screen to display some sort of cursor or crosshair
        egui::Window::new("Cursor")
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::none())
            .title_bar(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(
                    RichText::new("X")
                        .color(egui::Color32::from_rgb(255, 255, 255))
                        .heading(),
                );
            });

        egui::Window::new("Debug")
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .frame(egui::Frame::none())
            .title_bar(false)
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(10.0, 10.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!("FPS: {:?}", fps.unwrap_or_default().round()),
                );

                ui.separator();

                ui.heading(
                    RichText::new("Position")
                        .color(egui::Color32::from_rgb(255, 255, 255))
                        .heading(),
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!(
                        "World Position: X: {:?} Y: {:?} Z: {:?}",
                        player_pos.x, player_pos.y, player_pos.z
                    ),
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!("Chunk Position: X: {:?} Z: {:?}", chunk_pos.x, chunk_pos.z),
                );
                ui.colored_label(
                    egui::Color32::from_rgb(255, 255, 255),
                    format!(
                        "Local Position: X: {:?} Y: {:?} Z: {:?}",
                        local_pos.x, local_pos.y, local_pos.z
                    ),
                );

                if let Some(looking_at_pos) = player.looking_at_pos {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 255, 255),
                        format!(
                            "Looking At: X: {:?} Y: {:?} Z: {:?}",
                            looking_at_pos.x, looking_at_pos.y, looking_at_pos.z
                        ),
                    );
                }
            });
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(
                Update,
                debug_menu_system.run_if(in_state(ClientState::Playing)),
            )
            .add_systems(
                Update,
                loading_menu_system.run_if(
                    in_state(ClientState::LoadingTexture)
                        .or_else(in_state(ClientState::JoiningServer)),
                ),
            );
    }
}
