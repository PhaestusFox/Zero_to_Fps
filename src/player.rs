use core::f32;

use avian3d::prelude::*;
use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_editor_pls::{egui::widgets, EditorPlugin};
use leafwing_input_manager::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_systems(Startup, (spawn_player, lock_mouse))
        .add_systems(PreUpdate, update_grounded)
        .add_systems(
            Update,
            (
                apply_movement_damping,
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
pub struct PlayerCam;

#[derive(Component)]
pub struct Player;

#[derive(Reflect, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum PlayerAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Look,
    FlyUp,
    FlyDown,
    Shoot,
    Jump,
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
        (PlayerAction::Jump, KeyCode::Space),
    ])
    .with_dual_axis(PlayerAction::Look, MouseMove::default().sensitivity(0.1));
    map.insert(PlayerAction::FlyUp, KeyCode::Space)
        .insert(PlayerAction::FlyDown, KeyCode::ShiftLeft)
        .insert(PlayerAction::Shoot, MouseButton::Left);
    map
}

fn spawn_player(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let player = commands
        .spawn((
            Name::new("Player"),
            Player,
            SpatialBundle::default(),
            Collider::capsule(0.5, 1.),
            mesh_assets.add(Capsule3d::new(0.5, 1.)),
            material_assets.add(StandardMaterial::default()),
            ShapeCaster::new(
                Collider::capsule(0.5, 1.),
                Vec3::ZERO,
                Quat::IDENTITY,
                Dir3::new_unchecked(Vec3::NEG_Y),
            ),
            RigidBody::Dynamic,
            InputManagerBundle {
                input_map: player_bindings(),
                action_state: ActionState::default(),
            },
            MovementDampingFactor(0.9),
            LockedAxes::ROTATION_LOCKED,
            CollidingEntities::default(),
            CollisionLayers::new(
                super::Layers::Player,
                [super::Layers::Blasters, super::Layers::Ground],
            ), // ActiveEvents::all(),
               // CollisionGroups::new(Group::ALL, Group::ALL ^ Group::GROUP_2),
        ))
        .id();

    commands.entity(player).with_children(|p| {
        p.spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::Y * 0.5),
                ..Default::default()
            },
            RayCaster::new(Vec3::ZERO, Dir3::new(Vec3::NEG_Z).unwrap())
                .with_ignore_self(true)
                .with_max_time_of_impact(10.)
                .with_query_filter(SpatialQueryFilter::from_excluded_entities([player])),
            RayHits::default(),
            PlayerCam,
        ));
    });
}

fn player_move(
    mut player: Query<
        (
            &mut LinearVelocity,
            &Children,
            &ActionState<PlayerAction>,
            &RigidBody,
            Option<&Grounded>,
        ),
        With<Player>,
    >,
    camera: Query<&GlobalTransform, With<PlayerCam>>,
) {
    for (mut player, children, actions, body, ground) in &mut player {
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
        if body == &RigidBody::Kinematic {
            if actions.pressed(&PlayerAction::FlyDown) {
                delta.y -= 1.;
            }
            if actions.pressed(&PlayerAction::FlyUp) {
                delta.y += 1.;
            }
        }
        if ground.is_some() && actions.just_pressed(&PlayerAction::Jump) {
            delta.y += 10.;
        }
        player.0 += delta;
    }
}

fn player_look(
    window: Query<&Window, With<PrimaryWindow>>,
    mut player: Query<(&mut Transform, &Children, &ActionState<PlayerAction>), With<Player>>,
    mut camera: Query<&mut Transform, (With<PlayerCam>, Without<Player>)>,
) {
    let window = window.single();
    if !window.focused {
        return;
    }
    let scale = window.width().min(window.height()) / window.width();
    for (mut body, children, actions) in &mut player {
        let Some(child) = children.first().cloned() else {
            error!("Player has not child entity");
            continue;
        };
        let Ok(mut camera) = camera.get_mut(child) else {
            error!("first child is not camera");
            continue;
        };

        let look = actions.axis_pair(&PlayerAction::Look);

        let (_, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
        camera.rotation = Quat::from_axis_angle(
            Vec3::X,
            (pitch - (look.y * scale).to_radians())
                .clamp(-f32::consts::FRAC_PI_2, f32::consts::FRAC_PI_2),
        );
        let (yaw, _, _) = body.rotation.to_euler(EulerRot::YXZ);
        body.rotation = Quat::from_axis_angle(Vec3::Y, yaw - (look.x * scale).to_radians());
    }
}

fn noclip(mut player: Query<&mut RigidBody, With<Player>>) {
    for mut player in &mut player {
        match *player {
            RigidBody::Dynamic => *player = RigidBody::Kinematic,
            RigidBody::Kinematic => *player = RigidBody::Dynamic,
            _ => {}
        }
    }
}

#[derive(Component)]
struct MovementDampingFactor(f32);

fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}

#[derive(Component)]
struct Grounded;

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(mut commands: Commands, mut query: Query<(Entity, &ShapeHits), With<Player>>) {
    for (entity, hits) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|_| true);

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}
