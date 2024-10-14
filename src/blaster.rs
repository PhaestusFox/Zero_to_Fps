use core::f32;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    health::Health,
    map::Despawn,
    player::{Player, PlayerAction, PlayerCam},
    Layers,
};

pub fn plugin(app: &mut App) {
    app.add_event::<BlasterEvent>()
        .register_type::<Ammo>()
        .register_type::<Blaster>()
        .register_type::<Recoil>()
        .init_resource::<ShootSound>()
        .add_systems(Update, (equip_gun, fire, recoil, hit_scan))
        .add_systems(PostUpdate, pickup_gun_empty);
}

#[derive(Component, Reflect, serde::Deserialize)]
#[reflect(Deserialize, Component)]
pub struct Blaster;

#[derive(Component, Reflect, serde::Deserialize)]
#[reflect(Deserialize, Component)]
pub struct Ammo(u8);

#[derive(Component)]
pub struct CurrentBlaster(Entity);

fn pickup_gun_empty(
    mut context: EventReader<CollisionStarted>,
    mut commands: Commands,
    blasters: Query<Entity, With<Blaster>>,
    childern: Query<&Parent>,
    player: Query<Entity, (With<Player>, Without<CurrentBlaster>)>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    for colliding in context.read() {
        let colliding = if colliding.0 == player {
            colliding.1
        } else if colliding.1 == player {
            colliding.0
        } else {
            continue;
        };
        if blasters.get(colliding).is_ok() {
            commands.entity(colliding).remove::<RigidBody>();
            commands.entity(colliding).remove::<Collider>();
            commands.entity(colliding).insert((
                CollisionLayers::new(Layers::Blasters, Layers::all_bits()),
                RigidBody::Static,
            ));
            commands.entity(player).insert(CurrentBlaster(colliding));
            return;
        }
        if let Ok(parent) = childern.get(colliding) {
            if blasters.get(parent.get()).is_ok() {
                commands.entity(parent.get()).remove::<RigidBody>();
                commands.entity(colliding).remove::<Collider>();
                commands
                    .entity(colliding)
                    .insert((CollisionLayers::new(Layers::Blasters, Layers::all_bits()),));
                commands.entity(player).insert(CurrentBlaster(parent.get()));
                return;
            }
        }
    }
}

fn equip_gun(
    mut commands: Commands,
    player: Query<(Entity, &CurrentBlaster), Changed<CurrentBlaster>>,
    mut blaster: Query<(Entity, &mut Transform)>,
) {
    let Ok((player, equip)) = player.get_single() else {
        return;
    };
    let Ok((set, mut blaster)) = blaster.get_mut(equip.0) else {
        error!("Current blaster has no transform");
        return;
    };
    commands.entity(set).set_parent(player);
    *blaster = Transform::from_translation(Vec3::NEG_Z)
        .with_rotation(Quat::from_rotation_y(180f32.to_radians()));
}

#[derive(Component, Reflect, serde::Deserialize)]
#[reflect(Deserialize, Component)]
struct Recoil(f32);

#[derive(Resource)]
struct ShootSound(Vec<Handle<AudioSource>>);

impl ShootSound {
    fn get(&self) -> Handle<AudioSource> {
        use rand::seq::*;
        self.0.choose(&mut rand::thread_rng()).cloned().unwrap()
    }
}

impl FromWorld for ShootSound {
    fn from_world(world: &mut World) -> Self {
        let mut sounds = Vec::new();
        let server = world.resource::<AssetServer>();
        for i in 0..5 {
            sounds.push(server.load(format!("Sci-Fi-Sound/laserLarge_00{}.ogg", i)));
            sounds.push(server.load(format!("Sci-Fi-Sound/laserRetro_00{}.ogg", i)));
            sounds.push(server.load(format!("Sci-Fi-Sound/laserSmall_00{}.ogg", i)));
        }
        ShootSound(sounds)
    }
}

#[derive(Event)]
enum BlasterEvent {
    Fire,
}

fn fire(
    mut commands: Commands,
    mut blasters: Query<(Entity, &mut Recoil, &mut Ammo), With<Blaster>>,
    player: Query<(Entity, &ActionState<PlayerAction>, &CurrentBlaster)>,
    sound: Res<ShootSound>,
    mut blaster_event: EventWriter<BlasterEvent>,
) {
    let Ok((player_entity, player, gun)) = player.get_single() else {
        return;
    };
    if !player.just_pressed(&PlayerAction::Shoot) {
        return;
    }
    let Ok((blaster, mut recoil, mut ammo)) = blasters.get_mut(gun.0) else {
        return;
    };
    if recoil.0 > 0. {
        return;
    }
    blaster_event.send(BlasterEvent::Fire);
    recoil.0 += 1.;
    commands.entity(blaster).insert(AudioSourceBundle {
        source: sound.get(),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Remove,
            ..Default::default()
        },
    });
    if ammo.0 == 0 {
        commands
            .entity(blaster)
            .remove::<Blaster>()
            .remove_parent_in_place()
            .insert((
                RigidBody::Dynamic,
                LinearVelocity(Vec3::new(0., 5., -1.5)),
                AngularVelocity(Vec3::Y),
                Despawn::new(5.),
            ));
        commands.entity(player_entity).remove::<CurrentBlaster>();
    } else {
        ammo.0 -= 1;
    }
}

fn recoil(mut blasters: Query<(&mut Transform, &mut Recoil)>, time: Res<Time>) {
    for (mut pos, mut recoil) in &mut blasters {
        if recoil.0 <= 0. {
            continue;
        }
        let (_, yaw, roll) = pos.rotation.to_euler(EulerRot::XYZ);
        recoil.0 -= time.delta_seconds() * 3.;
        pos.rotation = Quat::from_euler(EulerRot::XYZ, recoil.0 + f32::consts::PI, yaw, roll);
    }
}

fn hit_scan(
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut objects: Query<(Entity, &mut Health)>,
    parents: Query<&Parent>,
    mut blaster_event: EventReader<BlasterEvent>,
    player: Query<(&Parent, &GlobalTransform, &RayHits), With<PlayerCam>>,
) {
    let (entity, player, rays) = player.single();
    for event in blaster_event.read() {
        match event {
            BlasterEvent::Fire => {
                gizmos.line(
                    player.translation(),
                    player.translation() + player.forward().as_vec3() * 10.,
                    bevy::color::palettes::basic::RED,
                );
                for hit in rays.iter() {
                    if let Ok((object, mut health)) = objects.get_mut(hit.entity) {
                        if health.0 > 1 {
                            health.0 -= 1;
                        } else {
                            commands.entity(object).remove::<Health>();
                        }
                        continue;
                    }
                    let Ok(parent) = parents.get(hit.entity) else {
                        continue;
                    };
                    if let Ok((object, mut health)) = objects.get_mut(parent.get()) {
                        if health.0 > 1 {
                            health.0 -= 1;
                        } else {
                            commands.entity(object).remove::<Health>();
                        }
                        continue;
                    }
                }
            }
        }
    }
}
