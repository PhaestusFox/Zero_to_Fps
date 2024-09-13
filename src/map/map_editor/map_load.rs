use bevy::prelude::*;

use crate::{
    map::{MapCellBundle, MapRoot},
    player::Player,
};

use super::MapData;

pub fn plugin(app: &mut App) {
    app.init_state::<MapLoadState>()
        .insert_resource(LoadMap("Maps/menu.map".to_string()))
        .init_resource::<CurrentMap>()
        .add_systems(
            Update,
            (
                load_map.run_if(in_state(MapLoadState::Loaded)),
                wait_for_loading.run_if(in_state(MapLoadState::Loading)),
            ),
        )
        .add_systems(OnEnter(MapLoadState::Spawning), (spawn_map, reset_player));
}

#[derive(Resource, Default)]
pub struct CurrentMap(pub Handle<MapData>);

fn load_map(
    map_to_load: Res<LoadMap>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<MapLoadState>>,
    mut current: ResMut<CurrentMap>,
) {
    if !map_to_load.is_changed() {
        return;
    }
    current.0 = asset_server.load(&map_to_load.0);
    next.set(MapLoadState::Loading);
}

fn wait_for_loading(
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<MapLoadState>>,
    current: Res<CurrentMap>,
) {
    if asset_server.is_loaded_with_dependencies(current.0.id()) {
        next.set(MapLoadState::Spawning)
    }
}

fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current: Res<CurrentMap>,
    map_data: Res<Assets<MapData>>,
    mut next: ResMut<NextState<MapLoadState>>,
) {
    let Some(data) = map_data.get(current.0.id()) else {
        error!("Map should be loaded");
        return;
    };
    commands
        .spawn((Name::new("Map Root"), SpatialBundle::default(), MapRoot))
        .with_children(|p| {
            for tile in data.tiles.iter() {
                p.spawn(MapCellBundle {
                    transform: tile.transform,
                    cell: asset_server.load(format!("Cells/{}", tile.cell)),
                    ..Default::default()
                });
            }
        });
    next.set(MapLoadState::Loaded)
}

fn reset_player(
    mut players: Query<&mut Transform, With<Player>>,
    current: Res<CurrentMap>,
    map_data: Res<Assets<MapData>>,
) {
    let Some(data) = map_data.get(current.0.id()) else {
        error!("Map should be loaded");
        return;
    };
    for mut player in &mut players {
        *player = data.spawn;
    }
}

#[derive(Resource)]
pub struct LoadMap(pub String);

#[derive(States, Clone, Debug, Hash, PartialEq, Eq, Default)]
enum MapLoadState {
    Loading,
    Spawning,
    #[default]
    Loaded,
}
