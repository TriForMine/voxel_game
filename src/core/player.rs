use crate::voxel::quad::HALF_SIZE;
use crate::voxel::world::World;
use crate::{AppState, GameWorld};
use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};

pub const PLAYER_HEIGHT: f32 = 1.8;
pub const CAMERA_HEIGHT: f32 = PLAYER_HEIGHT - 0.3;
pub const PLAYER_WIDTH: f32 = 0.5;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Default)]
pub struct VerticalMomentum(pub f32);

#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
    pub jump_height: f32,
    pub gravity: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 6.,
            jump_height: 5.,
            gravity: 9.8,
        }
    }
}

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub jump: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::Z,
            move_backward: KeyCode::S,
            move_left: KeyCode::Q,
            move_right: KeyCode::D,
            jump: KeyCode::Space,
            toggle_grab_cursor: KeyCode::Escape,
        }
    }
}

fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn setup_player(mut commands: Commands, game_world: Res<GameWorld>) {
    let highest_block = game_world
        .world
        .get_highest_block_at_coord(&IVec2::new(0, 0))
        .as_vec3();

    commands
        .spawn((
            Player,
            TransformBundle::from(
                Transform::from_xyz(
                    highest_block.x,
                    highest_block.y + HALF_SIZE + 2.,
                    highest_block.z,
                )
                .looking_to(Vec3::Z, Vec3::Y),
            ),
            VerticalMomentum(0.),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0., CAMERA_HEIGHT, 0.)
                        .looking_to(Vec3::Z, Vec3::Y),
                    ..Default::default()
                },
                PlayerCamera,
            ));
        });
}

fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<(&mut Transform, &mut VerticalMomentum), With<Player>>,
    game_world: Res<GameWorld>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (mut transform, mut vertical_momentum) in query.iter_mut() {
            let mut desired_velocity = Vec3::ZERO;

            // Check if the player is on the ground
            let is_grounded = game_world
                .world
                .check_block_at_coord(&World::coord_to_world(
                    transform.translation
                        - Vec3::new(
                            -PLAYER_WIDTH,
                            settings.gravity * time.delta_seconds(),
                            -PLAYER_WIDTH,
                        ),
                ))
                || game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            - Vec3::new(
                                PLAYER_WIDTH,
                                settings.gravity * time.delta_seconds(),
                                -PLAYER_WIDTH,
                            ),
                    ))
                || game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            - Vec3::new(
                                PLAYER_WIDTH,
                                settings.gravity * time.delta_seconds(),
                                PLAYER_WIDTH,
                            ),
                    ))
                || game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            - Vec3::new(
                                PLAYER_WIDTH,
                                settings.gravity * time.delta_seconds(),
                                -PLAYER_WIDTH,
                            ),
                    ));

            for key in keys.get_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let key = *key;
                        if key == key_bindings.move_forward {
                            desired_velocity += transform.forward() * settings.speed;
                        } else if key == key_bindings.move_backward {
                            desired_velocity += transform.back() * settings.speed;
                        } else if key == key_bindings.move_left {
                            desired_velocity += transform.left() * settings.speed;
                        } else if key == key_bindings.move_right {
                            desired_velocity += transform.right() * settings.speed;
                        }
                    }
                }
            }

            for key in keys.get_just_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let key = *key;
                        if key == key_bindings.jump && is_grounded {
                            vertical_momentum.0 = settings.jump_height;
                        }
                    }
                }
            }

            if vertical_momentum.0 > -settings.gravity {
                vertical_momentum.0 = (vertical_momentum.0
                    - settings.gravity * time.delta_seconds())
                .max(-settings.gravity);
            }

            desired_velocity.y += vertical_momentum.0;

            if desired_velocity.y < 0. && is_grounded {
                desired_velocity.y = 0.;
            }

            // Check front
            if desired_velocity.z > 0.
                && (game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation + Vec3::new(0., 0., PLAYER_WIDTH),
                    ))
                    || game_world
                        .world
                        .check_block_at_coord(&World::coord_to_world(
                            transform.translation + Vec3::new(0., 1., PLAYER_WIDTH),
                        )))
            {
                desired_velocity.z = 0.;
            }

            // Check back
            if desired_velocity.z < 0.
                && (game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation + Vec3::new(0., 0., -PLAYER_WIDTH),
                    ))
                    || game_world
                        .world
                        .check_block_at_coord(&World::coord_to_world(
                            transform.translation + Vec3::new(0., 1., -PLAYER_WIDTH),
                        )))
            {
                desired_velocity.z = 0.;
            }

            // Check right
            if desired_velocity.x > 0.
                && (game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation + Vec3::new(PLAYER_WIDTH, 0., 0.),
                    ))
                    || game_world
                        .world
                        .check_block_at_coord(&World::coord_to_world(
                            transform.translation + Vec3::new(PLAYER_WIDTH, 1., 0.),
                        )))
            {
                desired_velocity.x = 0.;
            }

            // Check left
            if desired_velocity.x < 0.
                && (game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation + Vec3::new(-PLAYER_WIDTH, 0., 0.),
                    ))
                    || game_world
                        .world
                        .check_block_at_coord(&World::coord_to_world(
                            transform.translation + Vec3::new(-PLAYER_WIDTH, 1., 0.),
                        )))
            {
                desired_velocity.x = 0.;
            }

            // Check top
            if desired_velocity.y > 0.
                && game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            + Vec3::new(
                                -PLAYER_WIDTH,
                                PLAYER_HEIGHT + settings.jump_height * time.delta_seconds(),
                                -PLAYER_WIDTH,
                            ),
                    ))
                || game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            + Vec3::new(
                                PLAYER_WIDTH,
                                PLAYER_HEIGHT + settings.jump_height * time.delta_seconds(),
                                -PLAYER_WIDTH,
                            ),
                    ))
                || game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            + Vec3::new(
                                PLAYER_WIDTH,
                                PLAYER_HEIGHT + settings.jump_height * time.delta_seconds(),
                                PLAYER_WIDTH,
                            ),
                    ))
                || game_world
                    .world
                    .check_block_at_coord(&World::coord_to_world(
                        transform.translation
                            + Vec3::new(
                                PLAYER_WIDTH,
                                PLAYER_HEIGHT + settings.jump_height * time.delta_seconds(),
                                -PLAYER_WIDTH,
                            ),
                    ))
            {
                desired_velocity.y = 0.;
            }

            transform.translation += desired_velocity * time.delta_seconds();
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&Parent, &mut Transform), (With<PlayerCamera>, Without<Player>)>,
    mut parent_transform_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (parent, mut transform) in query.iter_mut() {
            for ev in state.reader_motion.iter(&motion) {
                let mut parent_transform = parent_transform_query.get_mut(parent.get()).unwrap();

                let (_, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                let (mut yaw, _, _) = parent_transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                parent_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw);

                transform.rotation = Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(key_bindings.toggle_grab_cursor) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

fn initial_grab_on_player_spawn(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    query_added: Query<Entity, Added<Player>>,
) {
    if query_added.is_empty() {
        return;
    }

    if let Ok(window) = &mut primary_window.get_single_mut() {
        toggle_grab_cursor(window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .init_resource::<KeyBindings>()
            .add_systems(OnEnter(AppState::Playing), setup_player)
            .add_systems(
                Update,
                (
                    initial_grab_on_player_spawn,
                    player_move,
                    player_look,
                    cursor_grab,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}