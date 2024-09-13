use core::f32;

use bevy::{
    asset::LoadedFolder,
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_editor_pls::{egui::widgets, EditorPlugin};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

mod player;

mod map;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, RapierPhysicsPlugin::<()>::default()))
        .add_systems(Startup, (spawn_world, test_spawn))
        .add_plugins((player::plugin, map::plugin));
    #[cfg(debug_assertions)]
    app.add_plugins((EditorPlugin::new(), RapierDebugRenderPlugin::default()));
    app.run();
}

fn spawn_world(mut commands: Commands) {
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_translation(Vec3::NEG_Y * 10.),
            ..Default::default()
        },
        Collider::cuboid(10., 0.5, 10.),
        RigidBody::Fixed,
    ));
}

fn test_spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    for i in 0..'R' as u32 - 'A' as u32 {
        let scene = asset_server.load(format!(
            "Blasters/blaster{}.glb#Scene0",
            char::from_u32('A' as u32 + i).unwrap_or('A')
        ));
        commands.spawn(SceneBundle {
            scene,
            transform: Transform::from_translation(Vec3::Z * i as f32),
            ..Default::default()
        });
    }
    let Ok(dir) = std::fs::read_dir("assets/Conveyor") else {
        println!("Failed to read dir");
        return;
    };
    let mut x = 0.;
    let mut z = 4.;
    for path in dir {
        let Ok(dir) = path else {
            continue;
        };
        commands.spawn(SceneBundle {
            scene: asset_server.load(format!(
                "Conveyor/{}#Scene0",
                dir.path().file_name().unwrap().to_str().unwrap()
            )),
            transform: Transform::from_translation(Vec3::new(x, 0., z)),
            ..Default::default()
        });
        x += 2.;
        if x > 20. {
            x = 0.;
            z += 2.;
        }
    }
}
