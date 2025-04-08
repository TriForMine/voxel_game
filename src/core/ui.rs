use crate::core::player::{Player, PlayerCamera};
use crate::voxel::world::World;
use crate::{new_renet_client, new_renet_server, ClientMode, ClientState, ServerState};
use bevy::app::{App, AppExit};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::math::IVec3;
use bevy::prelude::*;
use bevy_egui::egui::RichText;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_renet::renet::RenetClient;
use renet_visualizer::RenetServerVisualizer;
use std::net::SocketAddr;
use std::time::Duration;

// Store main menu current menu state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum MainMenuState {
    #[default]
    MainMenu,
    Singleplayer,
    Multiplayer,
    Settings,
}

fn loading_menu_system(mut contexts: EguiContexts) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.heading("Loading");
    });
}

#[derive(Default)]
struct MultiplayerMenuState {
    server_ip: String,
}

fn main_menu_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    state: Res<State<MainMenuState>>,
    mut next_client_mode_state: ResMut<NextState<ClientMode>>,
    mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
    mut next_server_state: ResMut<NextState<ServerState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    mut exit: EventWriter<AppExit>,
    mut multiplayer_menu_state: Local<MultiplayerMenuState>,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| match state.get() {
        MainMenuState::MainMenu => {
            ui.heading("Main Menu");

            if ui.button("Singleplayer").clicked() {
                next_main_menu_state.set(MainMenuState::Singleplayer);
            }

            if ui.button("Multiplayer").clicked() {
                next_main_menu_state.set(MainMenuState::Multiplayer);
            }

            if ui.button("Settings").clicked() {
                next_main_menu_state.set(MainMenuState::Settings);
            }

            if ui.button("Quit").clicked() {
                exit.send(AppExit::Success);
            }
        }
        MainMenuState::Singleplayer => {
            ui.heading("Singleplayer");

            if ui.button("Create World").clicked() {
                // Create a server
                let (server, server_transport, addr) = new_renet_server(true);

                commands.insert_resource(server);
                commands.insert_resource(server_transport);
                commands.insert_resource(RenetServerVisualizer::<200>::default());

                next_server_state.set(ServerState::LoadingWorld);

                let (client, transport) = new_renet_client(addr);
                commands.insert_resource(client);
                commands.insert_resource(transport);

                next_client_state.set(ClientState::JoiningServer);
                next_main_menu_state.set(MainMenuState::MainMenu);
                next_client_mode_state.set(ClientMode::SinglePlayer);
            }

            if ui.button("Back").clicked() {
                next_main_menu_state.set(MainMenuState::MainMenu);
            }
        }
        MainMenuState::Multiplayer => {
            ui.heading("Multiplayer");

            // Enter server ip
            ui.horizontal(|ui| {
                ui.label("Server IP:");
                ui.text_edit_singleline(&mut multiplayer_menu_state.server_ip);
            });

            // Connect to server
            if ui.button("Connect").clicked() {
                let server_addr = if multiplayer_menu_state.server_ip.contains(':') {
                    multiplayer_menu_state.server_ip.parse().unwrap()
                } else {
                    SocketAddr::new(multiplayer_menu_state.server_ip.parse().unwrap(), 5000)
                };

                println!("Connecting to server: {}", server_addr);

                let (client, transport) = new_renet_client(server_addr);
                commands.insert_resource(client);
                commands.insert_resource(transport);

                next_client_state.set(ClientState::JoiningServer);
                next_main_menu_state.set(MainMenuState::MainMenu);
                next_client_mode_state.set(ClientMode::Online);
            }

            if ui.button("Back").clicked() {
                next_main_menu_state.set(MainMenuState::MainMenu);
            }
        }
        MainMenuState::Settings => {
            ui.heading("Settings");

            if ui.button("Back").clicked() {
                next_main_menu_state.set(MainMenuState::MainMenu);
            }
        }
    });
}

fn server_loading_menu_system(
    client: Res<RenetClient>,
    mut contexts: EguiContexts,
    mut next_server_state: ResMut<NextState<ServerState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
    mut timeout: Local<Timer>,
    time: Res<Time>,
) {
    timeout.set_duration(Duration::from_secs(15));

    if client.is_connected() {
        timeout.reset();
        next_client_state.set(ClientState::LoadingWorld);
    } else if let Some(reason) = client.disconnect_reason() {
        egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Failed to connect to server: {:?}", reason));

            if ui.button("Back").clicked() {
                timeout.reset();

                next_server_state.set(ServerState::MainMenu);
                next_client_state.set(ClientState::MainMenu);
            }
        });
    } else {
        timeout.tick(time.delta());
        egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
            ui.heading("Joining Server");

            if timeout.finished() {
                ui.label("Failed to connect to server");
                ui.label("Timed out");
            } else {
                ui.label(format!(
                    "Timeout: {:.2}",
                    timeout.duration().as_secs_f32() - timeout.elapsed().as_secs_f32()
                ));
            }

            if ui.button("Back").clicked() {
                timeout.reset();

                next_server_state.set(ServerState::MainMenu);
                next_client_state.set(ClientState::MainMenu);
            }
        });
    }
}

fn debug_menu_system(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    player_query: Query<(&Player, &Transform), (With<Player>, Without<PlayerCamera>)>,
) {
    if let Ok((player, player_transform)) = player_query.get_single() {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
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
                server_loading_menu_system.run_if(in_state(ClientState::JoiningServer)),
            )
            .add_systems(
                Update,
                main_menu_system.run_if(in_state(ClientState::MainMenu)),
            )
            .add_systems(
                Update,
                loading_menu_system.run_if(
                    in_state(ClientState::LoadingTexture).or(in_state(ClientState::LoadingWorld)),
                ),
            );
    }
}
