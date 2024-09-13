use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Asset, Reflect)]
struct Cell {
    scene: Handle<Scene>,
    #[reflect(ignore)]
    collider: Collider,
}
