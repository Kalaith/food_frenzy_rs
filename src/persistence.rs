//! Cross-platform persistence helpers for the game.

use crate::state::{GameState, GuestState, ProgressionState, Timers};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

const SAVE_VERSION: u32 = 1;
const SAVE_KEY: &str = "feast-frenzy-save.json";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_DIR_NAME: &str = "WebHatchery";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE_NAME: &str = "food_frenzy";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE_EXT: &str = "json";

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

    save_json(SAVE_KEY, &snapshot)
}

pub fn load_game() -> Result<Option<FoodFrenzySave>, String> {
    if !exists(SAVE_KEY) {
        return Ok(None);
    }

    load_json::<FoodFrenzySave>(SAVE_KEY).map(Some)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_path() -> Result<PathBuf, String> {
    if let Some(path) = test_save_path_override() {
        return Ok(path);
    }

    let mut path = dirs::data_dir()
        .or_else(dirs::document_dir)
        .or_else(current_dir)
        .ok_or_else(|| "Failed to resolve save directory".to_string())?;

    path.push(SAVE_DIR_NAME);
    path.push("game_apps");
    path.push("food_frenzy");
    std::fs::create_dir_all(&path).map_err(|error| {
        format!(
            "Failed to create save directory {}: {error}",
            path.display()
        )
    })?;
    path.push(format!("{SAVE_FILE_NAME}.{SAVE_FILE_EXT}"));
    Ok(path)
}

#[cfg(not(target_arch = "wasm32"))]
fn test_save_path_override() -> Option<PathBuf> {
    #[cfg(test)]
    {
        std::env::var_os("FOOD_FRENZY_TEST_SAVE_PATH").map(PathBuf::from)
    }

    #[cfg(not(test))]
    {
        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn current_dir() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

fn save_json<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Failed to serialize save data: {error}"))?;

    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::wasm_storage::storage_set(key, &serialized);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = path_from_key(key)?;
        std::fs::write(&path, serialized)
            .map_err(|error| format!("Failed to write save file '{}': {error}", path.display()))
    }
}

fn load_json<T: serde::de::DeserializeOwned>(key: &str) -> Result<T, String> {
    #[cfg(target_arch = "wasm32")]
    let serialized = macroquad_toolkit::wasm_storage::storage_get(key)
        .ok_or_else(|| format!("No browser save data found for '{key}'."))?;

    #[cfg(not(target_arch = "wasm32"))]
    let serialized = {
        let path = path_from_key(key)?;
        std::fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read save file '{}': {error}", path.display()))?
    };

    serde_json::from_str(&serialized).map_err(|error| format!("Failed to parse save data: {error}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn path_from_key(key: &str) -> Result<PathBuf, String> {
    match key {
        SAVE_KEY => save_path(),
        _ => Err(format!("Unknown native save key: {key}")),
    }
}

fn exists(key: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        macroquad_toolkit::wasm_storage::storage_exists(key)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(path) = path_from_key(key) {
            std::path::Path::new(&path).exists()
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
