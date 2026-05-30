use crate::data::GameData;
use macroquad::prelude::*;
use std::collections::HashMap;

const TITLE_TEXTURE_PATHS: [&str; 2] = [
    "assets/images/food_frenzy_title.png",
    "food_frenzy_title.png",
];

pub async fn load_character_textures(data: &GameData) -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    for customer_type in &data.customer_types {
        let path = format!("assets/images/characters/{}.png", customer_type.id);
        if let Ok(texture) = load_texture(&path).await {
            texture.set_filter(FilterMode::Linear);
            textures.insert(customer_type.id.clone(), texture);
        }
    }

    textures
}

pub async fn load_title_texture() -> Option<Texture2D> {
    for path in TITLE_TEXTURE_PATHS {
        if let Ok(texture) = load_texture(path).await {
            texture.set_filter(FilterMode::Linear);
            return Some(texture);
        }
    }

    None
}
