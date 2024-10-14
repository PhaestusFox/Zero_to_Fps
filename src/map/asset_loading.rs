use core::f32;

use avian3d::prelude::*;
use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
    render::render_asset::RenderAssetUsages,
    utils::HashMap,
};

use super::{Cell, TileDirection};

#[derive(serde::Serialize, serde::Deserialize)]
enum ColliderAsset {
    Cuboid(Vec3),
    Mesh {
        vertexs: Vec<[f32; 3]>,
        indices: Vec<u16>,
    },
}

impl From<ColliderAsset> for Collider {
    fn from(value: ColliderAsset) -> Self {
        match value {
            ColliderAsset::Cuboid(size) => Collider::cuboid(size.x / 2., size.y / 2., size.z / 2.),
            ColliderAsset::Mesh { vertexs, indices } => {
                let mut mesh = Mesh::new(
                    bevy::render::mesh::PrimitiveTopology::TriangleList,
                    RenderAssetUsages::all(),
                );

                mesh.insert_indices(bevy::render::mesh::Indices::U16(indices));
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertexs);
                Collider::trimesh_from_mesh(&mesh).unwrap()
            }
        }
    }
}

#[test]
fn print_cell() {
    // let mut mesh = Mesh::new(
    //     bevy::render::mesh::PrimitiveTopology::TriangleList,
    //     RenderAssetUsages::all(),
    // );

    // mesh.insert_indices(bevy::render::mesh::Indices::U16(indices));
    // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex);
    let vertexs = vec![
        [0.25, 0., 0.5],
        [-0.25, 0., 0.5],
        [0.25, 0., 0.0],
        [-0.25, 0., 0.0],
        [0.25, 4., -0.5],
        [-0.25, 4., -0.5],
        [0.25, 4., 0.0],
        [-0.25, 4., 0.0],
    ];
    let indices = vec![0, 1, 2, 2, 3, 0, 2, 3, 6, 6, 7, 2, 4, 5, 6, 6, 7, 4];

    println!(
        "{:#?}",
        ron::to_string(&CellAsset {
            scene: "Conveyor/box-small.glb".to_string(),
            collider: ColliderAsset::Mesh { vertexs, indices },
            body: RigidBody::Dynamic,
            scale: 1.,
            collider_offset: None,
            // can_tile: TileDirection::X | TileDirection::Z,
            components: HashMap::default(),
            layer: None,
        })
    );
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CellAsset {
    scene: String,
    collider: ColliderAsset,
    #[serde(default)]
    collider_offset: Option<Vec3>,
    #[serde(default = "fixed")]
    body: RigidBody,
    #[serde(default = "one")]
    scale: f32,
    // can_tile: TileDirection,
    #[serde(default)]
    components: HashMap<String, String>,
    #[serde(default)]
    layer: Option<(u32, u32)>,
}

fn one() -> f32 {
    1.
}

fn fixed() -> RigidBody {
    RigidBody::Static
}

pub(super) struct CellAssetLoader(AppTypeRegistry);

impl FromWorld for CellAssetLoader {
    fn from_world(world: &mut World) -> Self {
        CellAssetLoader(world.resource::<AppTypeRegistry>().clone())
    }
}

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
        load_cell_asset(&self.0, reader, load_context)
    }
}

async fn load_cell_asset<'a>(
    registry: &AppTypeRegistry,
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

    let mut components = Vec::new();

    let registry = registry.read();
    for (type_name, data) in cell.components {
        let type_data = match registry.get_with_short_type_path(&type_name) {
            Some(data) => data,
            None => match registry.get_with_type_path(&type_name) {
                Some(data) => data,
                None => {
                    error!("Type({}) not in registry", type_name);
                    continue;
                }
            },
        };

        let Some(serde_data) = type_data.data::<ReflectDeserialize>() else {
            error!("Type({}) does not reflect deserialize", type_name);
            continue;
        };
        let Ok(mut de) = ron::Deserializer::from_str(&data) else {
            error!("Ron could not make deserializer for Type({})", type_name);
            continue;
        };
        let Ok(component) = serde_data.deserialize(&mut de) else {
            error!(
                "Failed to deserialize data({}) for Type({})",
                data, type_name
            );
            continue;
        };
        components.push(component);
    }

    let cell = Cell {
        scene: load_context.load(cell.scene),
        collider: cell.collider.into(),
        collider_offset: cell.collider_offset,
        body: cell.body,
        scale: cell.scale,
        // can_tile: cell.can_tile,
        components,
        layer: cell.layer,
    };
    Ok(cell)
}
