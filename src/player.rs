use core::f32;

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_editor_pls::{egui::widgets, EditorPlugin};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_systems(Startup, (spawn_player, lock_mouse))
        .add_systems(
            Update,
            (
                player_move,
                player_look,
                toggle_mouse.run_if(input_just_pressed(KeyCode::Escape)),
                noclip.run_if(input_just_pressed(KeyCode::F11)),
            ),
        );
}

fn lock_mouse(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    for mut window in &mut window {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }
}

fn toggle_mouse(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    for mut window in &mut window {
        match window.cursor.grab_mode {
            CursorGrabMode::None => {
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }
            CursorGrabMode::Confined | CursorGrabMode::Locked => {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        }
    }
}

#[derive(Component)]
struct PlayerCam;

#[derive(Component)]
pub struct Player;

#[derive(Reflect, Clone, Copy, Hash, PartialEq, Eq, Debug)]
enum PlayerAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Look,
    FlyUp,
    FlyDown,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::Look => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

fn player_bindings() -> InputMap<PlayerAction> {
    let mut map = InputMap::new([
        (PlayerAction::MoveUp, KeyCode::KeyW),
        (PlayerAction::MoveDown, KeyCode::KeyS),
        (PlayerAction::MoveLeft, KeyCode::KeyA),
        (PlayerAction::MoveRight, KeyCode::KeyD),
    ])
    .with_dual_axis(PlayerAction::Look, MouseMove::default().sensitivity(0.1));
    map.insert(PlayerAction::FlyUp, KeyCode::Space)
        .insert(PlayerAction::FlyDown, KeyCode::ShiftLeft);
    map
}

fn spawn_player(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn((
            Player,
            SpatialBundle::default(),
            Collider::capsule(Vec3::Y, Vec3::NEG_Y, 1.),
            mesh_assets.add(Capsule3d::new(0.5, 1.)),
            material_assets.add(StandardMaterial::default()),
            KinematicCharacterController::default(),
            RigidBody::Dynamic,
            InputManagerBundle {
                input_map: player_bindings(),
                action_state: ActionState::default(),
            },
            LockedAxes::ROTATION_LOCKED,
        ))
        .with_children(|p| {
            p.spawn((
                Camera3dBundle {
                    transform: Transform::from_translation(Vec3::Y * 0.5),
                    ..Default::default()
                },
                PlayerCam,
            ));
        });
}

fn player_move(
    mut player: Query<
        (
            &mut KinematicCharacterController,
            &Children,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
    camera: Query<&Transform, With<PlayerCam>>,
) {
    for (mut controller, children, actions) in &mut player {
        let mut delta = Vec3::default();
        if actions.pressed(&PlayerAction::MoveUp) {
            delta.z += 1.;
        }
        if actions.pressed(&PlayerAction::MoveDown) {
            delta.z -= 1.;
        }
        if actions.pressed(&PlayerAction::MoveLeft) {
            delta.x += 1.;
        }
        if actions.pressed(&PlayerAction::MoveRight) {
            delta.x -= 1.;
        }
        let Some(child) = children.first().cloned() else {
            error!("Player has not child entity");
            continue;
        };
        let Ok(camera) = camera.get(child) else {
            error!("first child is not camera");
            continue;
        };
        let forward = camera.forward().as_vec3() * delta.z;
        let left = camera.left().as_vec3() * delta.x;
        delta = forward + left;
        delta.y = 0.;
        #[cfg(debug_assertions)]
        if actions.pressed(&PlayerAction::FlyDown) {
            delta.y -= 1.;
        }
        if actions.pressed(&PlayerAction::FlyUp) {
            delta.y += 1.;
        }
        if let Some(current) = &mut controller.translation {
            *current += delta.normalize();
        } else {
            controller.translation = Some(delta.normalize());
        }
    }
}

fn player_look(
    player: Query<(&Children, &ActionState<PlayerAction>), With<Player>>,
    mut camera: Query<&mut Transform, With<PlayerCam>>,
) {
    for (children, actions) in &player {
        let Some(child) = children.first().cloned() else {
            error!("Player has not child entity");
            continue;
        };
        let Ok(mut camera) = camera.get_mut(child) else {
            error!("first child is not camera");
            continue;
        };

        let look = actions.axis_pair(&PlayerAction::Look);

        let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
        camera.rotation = Quat::from_axis_angle(Vec3::Y, yaw - (look.x).to_radians())
            * Quat::from_axis_angle(
                Vec3::X,
                (pitch - (look.y).to_radians())
                    .clamp(-f32::consts::FRAC_PI_2, f32::consts::FRAC_PI_2),
            );
    }
}

fn noclip(mut player: Query<&mut RigidBody, With<Player>>) {
    for mut player in &mut player {
        match *player {
            RigidBody::Dynamic => *player = RigidBody::Fixed,
            RigidBody::Fixed => *player = RigidBody::Dynamic,
            _ => {}
        }
    }
}
