use bevy::prelude::*;
use map_load::LoadMap;

mod asset_loading;
mod map_load;

#[derive(serde::Serialize, serde::Deserialize, Reflect)]
struct Tile {
    cell: String,
    transform: Transform,
}

#[derive(serde::Serialize, serde::Deserialize, Asset, Reflect)]
struct MapData {
    spawn: Transform,
    tiles: Vec<Tile>,
}

pub fn plugin(app: &mut App) {
    app.init_asset_loader::<asset_loading::MapLoader>()
        .init_asset::<MapData>()
        .add_plugins(map_load::plugin);
}

#[test]
fn print_mapdate() {
    let map = MapData {
        spawn: Transform::IDENTITY,
        tiles: vec![
            Tile {
                cell: String::from("floor.cell"),
                transform: Transform {
                    scale: Vec3::new(10., 1., 10.),
                    ..Default::default()
                },
            },
            Tile {
                cell: String::from("wall.cell"),
                transform: Transform {
                    scale: Vec3::new(20., 1., 1.),
                    translation: Vec3::new(0., 0., 10.),
                    ..Default::default()
                },
            },
            Tile {
                cell: String::from("wall.cell"),
                transform: Transform {
                    scale: Vec3::new(20., 1., 1.),
                    translation: Vec3::new(0., 0., -10.),
                    rotation: Quat::from_rotation_y(180f32.to_radians()),
                },
            },
            Tile {
                cell: String::from("wall.cell"),
                transform: Transform {
                    scale: Vec3::new(20., 1., 1.),
                    translation: Vec3::new(-10., 0., 0.),
                    rotation: Quat::from_rotation_y(90f32.to_radians()),
                },
            },
            Tile {
                cell: String::from("wall.cell"),
                transform: Transform {
                    scale: Vec3::new(20., 1., 1.),
                    translation: Vec3::new(10., 0., 0.),
                    rotation: Quat::from_rotation_y(-90f32.to_radians()),
                },
            },
        ],
    };
    println!(
        "{}",
        ron::ser::to_string_pretty(&map, ron::ser::PrettyConfig::default()).unwrap()
    )
}
