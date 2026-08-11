//! Cross-platform persistence helpers for the game.

use crate::state::{GameState, GuestState, ProgressionState, Timers};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Bump when the save schema changes; `migrate` upgrades older versions.
/// v1: pre-review-roadmap shape (bare-string plated dishes, no tutorial /
///     day-cycle / regulars / specialization state).
/// v2: the Phase 1-3 systems landed; new fields fill via serde defaults and
///     `PlatedDish` reads the legacy bare-string form.
const SAVE_VERSION: u32 = 2;
const GAME_NAME: &str = "feast_frenzy";
#[cfg(target_arch = "wasm32")]
const SAVE_KEY: &str = "feast-frenzy-save.json";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE_NAME: &str = "feast_frenzy.json";
#[cfg(not(target_arch = "wasm32"))]
const TEST_SAVE_PATH_ENV: &str = "feast_FRENZY_TEST_SAVE_PATH";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodFrenzySave {
    /// 0 = saves from before the version field existed.
    #[serde(default)]
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

    let mut save = load_json::<FoodFrenzySave>()?;
    migrate(&mut save);
    Ok(Some(save))
}

/// Upgrade an older save in place. Most version gaps are absorbed by serde
/// defaults and `PlatedDish`'s legacy deserializer; anything that needs an
/// explicit fixup gets a version arm here.
fn migrate(save: &mut FoodFrenzySave) {
    if save.version < 2 {
        // v0/v1 saves predate the day cycle: their run effectively resumes at
        // the start of a fresh service day rather than mid-ledger.
        save.game_state.day_cycle.summary_pending = false;
        save.game_state.day_cycle.elapsed_ms = 0.0;
    }
    save.version = SAVE_VERSION;
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
mod tests;
