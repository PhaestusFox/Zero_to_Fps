use core::f32;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::player::{Player, PlayerAction};

pub fn plugin(app: &mut App) {
    app.register_type::<Blaster>()
        .register_type::<Recoil>()
        .init_resource::<ShootSound>()
        .add_systems(Update, (pickup_gun_empty, equip_gun, fire, recoil));
}

#[derive(Component, Reflect, serde::Deserialize)]
#[reflect(Deserialize, Component)]
pub struct Blaster;

#[derive(Component)]
pub struct CurrentBlaster(Entity);

fn pickup_gun_empty(
    mut commands: Commands,
    blasters: Query<Entity, With<Blaster>>,
    childern: Query<&Parent>,
    player: Query<(Entity, &CollidingEntities), (With<Player>, Without<CurrentBlaster>)>,
) {
    let Ok((entity, colliding)) = player.get_single() else {
        return;
    };
    for colliding in colliding.iter() {
        if blasters.get(colliding).is_ok() {
            commands.entity(colliding).remove::<Collider>();
            commands.entity(entity).insert(CurrentBlaster(colliding));
            return;
        }
        if let Ok(parent) = childern.get(colliding) {
            if blasters.get(parent.get()).is_ok() {
                commands.entity(colliding).remove::<Collider>();
                commands.entity(parent.get()).remove::<RigidBody>();
                commands.entity(entity).insert(CurrentBlaster(parent.get()));
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

fn fire(
    mut commands: Commands,
    mut blasters: Query<(Entity, &mut Recoil), With<Blaster>>,
    player: Query<(&ActionState<PlayerAction>, &CurrentBlaster)>,
    sound: Res<ShootSound>,
) {
    let Ok((player, gun)) = player.get_single() else {
        return;
    };
    if !player.just_pressed(&PlayerAction::Shoot) {
        return;
    }
    let Ok((blaster, mut recoil)) = blasters.get_mut(gun.0) else {
        return;
    };
    if recoil.0 > 0. {
        return;
    }
    recoil.0 += 1.;
    commands.entity(blaster).insert(AudioSourceBundle {
        source: sound.get(),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Remove,
            ..Default::default()
        },
    });
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
