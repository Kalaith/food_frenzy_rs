use crate::data::GameData;
use macroquad::prelude::*;
use macroquad_toolkit::assets::{load_texture_from_pack_or_file, AssetPack};
use std::collections::HashMap;

const ASSET_PACK_PATH: &str = "assets.zip";

const TITLE_TEXTURE_PATHS: [&str; 2] = [
    "assets/images/food_frenzy_title.png",
    "food_frenzy_title.png",
];

pub async fn load_asset_pack() -> Option<AssetPack> {
    AssetPack::load(ASSET_PACK_PATH).await.ok()
}

pub async fn load_character_textures(
    data: &GameData,
    asset_pack: Option<&AssetPack>,
) -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    for customer_type in &data.customer_types {
        let path = format!("assets/images/characters/{}.png", customer_type.id);
        if let Ok(texture) =
            load_texture_from_pack_or_file(asset_pack, &path, FilterMode::Linear).await
        {
            textures.insert(customer_type.id.clone(), texture);
        }
    }

    textures
}

pub async fn load_title_texture(asset_pack: Option<&AssetPack>) -> Option<Texture2D> {
    for path in TITLE_TEXTURE_PATHS {
        if let Ok(texture) =
            load_texture_from_pack_or_file(asset_pack, path, FilterMode::Linear).await
        {
            return Some(texture);
        }
    }

    None
}
