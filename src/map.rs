use core::f32;

use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    render::render_asset::RenderAssetUsages,
};
use bevy_rapier3d::prelude::*;

mod asset_loading;
mod map_editor;

#[derive(Component)]
struct MapRoot;

pub fn plugin(app: &mut App) {
    app.init_asset_loader::<asset_loading::CellAssetLoader>()
        .init_asset::<Cell>()
        .add_systems(Startup, spawn_test_asset)
        .add_systems(Update, (onchange_cell, onload_cell))
        .add_systems(PostUpdate, add_dynamic_components)
        .add_plugins(map_editor::plugin);
}

#[derive(Asset, Reflect)]
pub struct Cell {
    scene: Handle<Scene>,
    #[reflect(ignore)]
    collider: Collider,
    collider_offset: Option<Vec3>,
    body: RigidBody,
    scale: f32,
    #[reflect(ignore)]
    can_tile: TileDirection,
    #[reflect(ignore)]
    components: Vec<Box<dyn Reflect>>,
}

use bitflags::bitflags;

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    #[derive(Default, serde::Serialize, serde::Deserialize)]
    /// Represents a set of flags.
    struct TileDirection: u8 {
        /// The value `A`, at bit position `0`.
        const X = 0b00000001;
        /// The value `B`, at bit position `1`.
        const Y = 0b00000010;
        /// The value `C`, at bit position `2`.
        const Z = 0b00000100;
    }
}

#[derive(Bundle, Default)]
struct MapCellBundle {
    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The inherited visibility of the entity.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility of the entity.
    pub view_visibility: ViewVisibility,
    /// The transform of the entity.
    pub transform: Transform,
    /// The global transform of the entity.
    pub global_transform: GlobalTransform,
    pub cell: Handle<Cell>,
}

fn spawn_test_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("small box"),
        MapCellBundle {
            cell: asset_server.load("Cells/box-small.cell"),
            transform: Transform::from_translation(Vec3::X * 2.),
            ..Default::default()
        },
    ));
    commands.spawn((
        Name::new("wide box"),
        MapCellBundle {
            cell: asset_server.load("Cells/box-wide.cell"),
            transform: Transform::from_translation(Vec3::X * 4.),
            ..Default::default()
        },
    ));
    commands.spawn((
        Name::new("long box"),
        MapCellBundle {
            cell: asset_server.load("Cells/box-long.cell"),
            transform: Transform::from_translation(Vec3::X * -2.),
            ..Default::default()
        },
    ));
    commands.spawn((
        Name::new("large box"),
        MapCellBundle {
            cell: asset_server.load("Cells/box-large.cell"),
            transform: Transform::from_translation(Vec3::X * -4.),
            ..Default::default()
        },
    ));
    commands.spawn((
        Name::new("Floor"),
        MapCellBundle {
            cell: asset_server.load("Cells/floor.cell"),
            transform: Transform::from_translation(Vec3::Y * -4.),
            ..Default::default()
        },
    ));
    commands.spawn((
        Name::new("Wall"),
        MapCellBundle {
            cell: asset_server.load("Cells/wall.cell"),
            transform: Transform::from_translation(Vec3::new(0., -4., 5.5)),
            ..Default::default()
        },
    ));
}

fn onload_cell(
    mut commands: Commands,
    mut asset_events: EventReader<AssetEvent<Cell>>,
    cells: Query<(Entity, &Handle<Cell>)>,
    cell_assets: Res<Assets<Cell>>,
) {
    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                for (cell, handle) in &cells {
                    if handle.id() == *id {
                        let Some(asset) = cell_assets.get(*id) else {
                            error!("Cell not in Assets<Cell> when loaded");
                            continue;
                        };
                        update_cell(&mut commands, cell, asset);
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Component)]
struct SetScale(f32);

fn onchange_cell(
    mut commands: Commands,
    cells: Query<(Entity, &Handle<Cell>), Changed<Handle<Cell>>>,
    cell_assets: Res<Assets<Cell>>,
) {
    for (cell, handle) in &cells {
        let Some(asset) = cell_assets.get(handle.id()) else {
            warn!("Cell not in Assets<Cell> when Changed");
            continue;
        };
        update_cell(&mut commands, cell, asset);
    }
}

fn update_scale(mut commands: Commands, mut objects: Query<(Entity, &mut Transform, &SetScale)>) {
    return;
    for (entity, mut transform, scale) in &mut objects {
        transform.scale = Vec3::splat(scale.0);
        commands.entity(entity).remove::<SetScale>();
    }
}

#[derive(Component)]
struct AddDynamicComponents;

fn update_cell(commands: &mut Commands, target: Entity, asset: &Cell) {
    let mut cell = commands.entity(target);
    cell.despawn_descendants();
    cell.remove::<Collider>();
    cell.insert((asset.scene.clone(), asset.body, SetScale(asset.scale)));
    if let Some(offset) = asset.collider_offset {
        cell.with_children(|p| {
            p.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(offset),
                    ..Default::default()
                },
                asset.collider.clone(),
            ));
        });
    } else {
        cell.insert(asset.collider.clone());
    }
    if !asset.components.is_empty() {
        cell.insert(AddDynamicComponents);
    }
}

fn add_dynamic_components(world: &mut World) {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let registry = registry.read();
    let query = QueryBuilder::<(Entity, &Handle<Cell>), With<AddDynamicComponents>>::new(world)
        .build()
        .iter(world)
        .map(|f| (f.0, f.1.clone_weak()))
        .collect::<Vec<_>>();
    world.resource_scope(|world: &mut World, cell_assets: Mut<Assets<Cell>>| {
        for (id, handel) in query {
            let Some(asset) = cell_assets.get(handel.id()) else {
                continue;
            };
            for component in asset.components.iter() {
                let Some(component_data) =
                    registry.get_type_data::<ReflectComponent>(component.type_id())
                else {
                    error!(
                        "type({}) not in registry",
                        std::any::type_name_of_val(component)
                    );
                    continue;
                };
                let mut entity = world.entity_mut(id);
                component_data.apply_or_insert(&mut entity, component.as_reflect(), &registry);
                entity.remove::<AddDynamicComponents>();
            }
        }
    })
}
