use bevy::prelude::*;
use rand::Rng;

use crate::map::{Cell, MapCellBundle};

pub fn plugin(app: &mut App) {
    app.register_type::<Health>()
        .register_type::<DropTable>()
        .add_systems(Update, (set_drops, spawn_drops));
}

#[derive(Component, Reflect, serde::Deserialize)]
#[reflect(Deserialize, Component)]
pub struct Health(pub u8);

#[derive(Component)]
pub struct Drops(Vec<(Vec3, Handle<Cell>)>);

#[derive(Component, serde::Deserialize, Reflect, serde::Serialize)]
#[reflect(Deserialize, Component)]
pub struct DropTable(Vec<Drop>);

#[derive(serde::Deserialize, Reflect, serde::Serialize)]
pub struct Drop {
    cell: String,
    chance: f32,
    min_amount: u8,
    max_amount: u8,
    offset: Vec3,
}

#[test]
fn drop_chance() {
    let drops = DropTable(vec![
        Drop {
            cell: "Cells/blaster-a.cell".to_string(),
            chance: 0.75,
            min_amount: 1,
            max_amount: 3,
            offset: Vec3::Y * 0.5,
        },
        Drop {
            cell: "Cells/blaster-b.cell".to_string(),
            chance: 0.25,
            min_amount: 1,
            max_amount: 1,
            offset: Vec3::Y * 0.5,
        },
    ]);
    println!(
        "{}",
        ron::ser::to_string_pretty(&drops, ron::ser::PrettyConfig::default()).unwrap()
    )
}

fn set_drops(
    mut commands: Commands,
    added: Query<(Entity, &DropTable), Added<DropTable>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, drops) in &added {
        println!("Set Drops for {:?}", entity);
        let mut drop = Vec::new();
        for item in drops.0.iter() {
            if rand::thread_rng().gen_bool(item.chance as f64) {
                let num = rand::thread_rng().gen_range(item.min_amount..item.max_amount);
                for _ in 0..num {
                    drop.push((item.offset, asset_server.load(&item.cell)))
                }
            }
        }
        if !drop.is_empty() {
            commands.entity(entity).insert(Drops(drop));
        } else {
            info!("{:?}: Not Drops Found", entity);
        }
    }
}

fn spawn_drops(
    mut commands: Commands,
    mut dead: RemovedComponents<Health>,
    drops: Query<(Entity, &Drops, &Transform)>,
) {
    for died in dead.read() {
        if let Ok((target, drops, pos)) = drops.get(died) {
            commands.entity(target).despawn_recursive();
            for drop in drops.0.iter() {
                let mut pos = *pos;
                pos.translation += drop.0;
                commands.spawn(MapCellBundle {
                    transform: pos,
                    cell: drop.1.clone(),
                    ..Default::default()
                });
            }
        }
    }
}
