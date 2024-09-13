use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    log::error,
};

use super::MapData;

#[derive(Default)]
pub(super) struct MapLoader;

impl AssetLoader for MapLoader {
    type Asset = MapData;

    type Settings = ();

    type Error = &'static str;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        load_map(reader)
    }
}

async fn load_map<'a>(
    reader: &'a mut bevy::asset::io::Reader<'_>,
) -> Result<MapData, &'static str> {
    let mut data = String::new();
    if reader.read_to_string(&mut data).await.is_err() {
        return Err("Failed to read to string");
    }
    match ron::from_str(&data) {
        Ok(ok) => Ok(ok),
        Err(e) => {
            error!("{}", e);
            Err("Ron Failed")
        }
    }
}
