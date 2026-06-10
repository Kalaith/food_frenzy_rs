//! Cross-platform persistence helpers for the game.

use crate::state::{GameState, GuestState, ProgressionState, Timers};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

const SAVE_VERSION: u32 = 1;
const GAME_NAME: &str = "food_frenzy";
#[cfg(target_arch = "wasm32")]
const SAVE_KEY: &str = "feast-frenzy-save.json";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE_NAME: &str = "food_frenzy.json";
#[cfg(not(target_arch = "wasm32"))]
const TEST_SAVE_PATH_ENV: &str = "FOOD_FRENZY_TEST_SAVE_PATH";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodFrenzySave {
    pub version: u32,
    pub game_state: GameState,
    pub progression_state: ProgressionState,
    pub guest_state: GuestState,
    pub timers: Timers,
    pub selected_station: Option<String>,
}

pub fn save_game(
    game_state: &GameState,
    progression_state: &ProgressionState,
    guest_state: &GuestState,
    timers: &Timers,
    selected_station: &Option<String>,
) -> Result<(), String> {
    let snapshot = FoodFrenzySave {
        version: SAVE_VERSION,
        game_state: game_state.clone(),
        progression_state: progression_state.clone(),
        guest_state: guest_state.clone(),
        timers: timers.clone(),
        selected_station: selected_station.clone(),
    };

    save_json(&snapshot)
}

pub fn load_game() -> Result<Option<FoodFrenzySave>, String> {
    if !save_exists() {
        return Ok(None);
    }

    load_json::<FoodFrenzySave>().map(Some)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_path() -> Result<PathBuf, String> {
    macroquad_toolkit::persistence::get_webhatchery_game_app_path(
        GAME_NAME,
        SAVE_FILE_NAME,
        Some(TEST_SAVE_PATH_ENV),
    )
    .ok_or_else(|| "Failed to resolve save directory".to_string())
}

fn save_json<T: Serialize>(value: &T) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::persistence::save_json_key(GAME_NAME, SAVE_KEY, value)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = save_path()?;
        macroquad_toolkit::persistence::save_json_atomic(&path, value)
            .map_err(|error| format!("Failed to write save file '{}': {error}", path.display()))
    }
}

fn load_json<T: serde::de::DeserializeOwned>() -> Result<T, String> {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::persistence::load_json_key(GAME_NAME, SAVE_KEY)
            .or_else(|_| load_legacy_browser_save())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = save_path()?;
        macroquad_toolkit::persistence::load_json(&path)
            .map_err(|error| format!("Failed to read save file '{}': {error}", path.display()))
    }
}

#[cfg(target_arch = "wasm32")]
fn load_legacy_browser_save<T: serde::de::DeserializeOwned>() -> Result<T, String> {
    let serialized = macroquad_toolkit::wasm_storage::storage_get(SAVE_KEY)
        .ok_or_else(|| format!("No browser save data found for '{SAVE_KEY}'."))?;
    let save = serde_json::from_str(&serialized)
        .map_err(|error| format!("Failed to parse save data: {error}"))?;
    let _ = macroquad_toolkit::persistence::save_string_key(GAME_NAME, SAVE_KEY, &serialized);
    Ok(save)
}

fn save_exists() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::persistence::json_key_exists(GAME_NAME, SAVE_KEY)
            || macroquad_toolkit::wasm_storage::storage_exists(SAVE_KEY)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(path) = save_path() {
            macroquad_toolkit::persistence::file_exists(path)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameData;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn can_save_and_load_round_trip() {
        let path =
            std::env::temp_dir().join(format!("food_frenzy_save_test_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        std::env::set_var("FOOD_FRENZY_TEST_SAVE_PATH", &path);

        let data = GameData::load();
        let game_state = crate::state::GameState::new(&data);
        let progression_state = crate::state::ProgressionState::from_game_data(&data);
        let guest_state = crate::state::GuestState::new();
        let timers = crate::state::Timers::new();
        let selected = None;

        let err = save_game(
            &game_state,
            &progression_state,
            &guest_state,
            &timers,
            &selected,
        );
        assert!(err.is_ok(), "save should succeed: {err:?}");

        let loaded = load_game().unwrap_or(None);
        assert!(loaded.is_some());

        std::env::remove_var("FOOD_FRENZY_TEST_SAVE_PATH");
        let _ = std::fs::remove_file(path);
    }
}
