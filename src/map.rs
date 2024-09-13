use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_asset_loader::<CellAssetLoader>()
        .init_asset::<Cell>()
        .add_systems(Startup, spawn_test_asset)
        .add_systems(Update, (onchange_cell, onload_cell));
}

#[derive(Asset, Reflect)]
struct Cell {
    scene: Handle<Scene>,
    #[reflect(ignore)]
    collider: Collider,
    collider_offset: Option<Vec3>,
}

// #[test]
// fn print_cell() {
//     println!(
//         "{:#?}",
//         ron::to_string(&CellAsset {
//             scene: "Conveyor/box-small.glb".to_string(),
//             collider: Collider::cuboid(0.5, 0.5, 0.5),
//         })
//     );
// }

#[derive(serde::Serialize, serde::Deserialize)]
struct CellAsset {
    scene: String,
    collider: Collider,
    #[serde(default)]
    collider_offset: Option<Vec3>,
}

#[derive(Default)]
struct CellAssetLoader;

impl AssetLoader for CellAssetLoader {
    type Asset = Cell;
    type Settings = ();
    type Error = &'static str;
    fn extensions(&self) -> &[&str] {
        &["cell"]
    }
    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        load_cell_asset(reader, load_context)
    }
}

async fn load_cell_asset<'a>(
    reader: &'a mut bevy::asset::io::Reader<'_>,
    load_context: &'a mut bevy::asset::LoadContext<'_>,
) -> Result<Cell, &'static str> {
    let mut data = String::new();
    if reader.read_to_string(&mut data).await.is_err() {
        return Err("Failed to read string");
    };
    let cell: CellAsset = match ron::from_str(&data) {
        Ok(cell) => cell,
        Err(e) => {
            error!("{}", e);
            return Err("Ron Failed");
        }
    };
    let cell = Cell {
        scene: load_context.load(cell.scene),
        collider: cell.collider,
        collider_offset: cell.collider_offset,
    };
    Ok(cell)
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
        Name::new("test box"),
        MapCellBundle {
            cell: asset_server.load("Cells/test.cell"),
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
                        let mut cell = commands.entity(cell);
                        cell.despawn_descendants();
                        cell.remove::<Collider>();
                        if let Some(offset) = asset.collider_offset {
                            cell.insert(asset.scene.clone()).with_children(|p| {
                                p.spawn((
                                    SpatialBundle {
                                        transform: Transform::from_translation(offset),
                                        ..Default::default()
                                    },
                                    asset.collider.clone(),
                                ));
                            });
                        } else {
                            cell.insert((asset.scene.clone(), asset.collider.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

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
        commands
            .entity(cell)
            .insert((asset.scene.clone(), asset.collider.clone()));
    }
}
