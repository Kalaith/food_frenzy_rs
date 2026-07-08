//! Sound effects, loaded via `macroquad_toolkit::audio`. Gameplay code queues
//! `SfxCue`s on `GameState` (it has no audio access); the app drains the queue
//! each frame and plays through the bank. All clips live in `assets/sounds/`
//! and load from the asset pack with a loose-file fallback; a missing clip
//! degrades to silence rather than failing the boot.

use crate::state::SfxCue;
use macroquad::audio::{play_sound, PlaySoundParams, Sound};
use macroquad_toolkit::assets::AssetPack;
use macroquad_toolkit::audio::load_sound_from_pack_or_file;
use std::collections::HashMap;

pub struct AudioBank {
    sounds: HashMap<SfxCue, Sound>,
    base_volume: f32,
    pub enabled: bool,
}

const CLIPS: [(SfxCue, &str); 7] = [
    (SfxCue::CookStart, "assets/sounds/cook_start.wav"),
    (SfxCue::DishReady, "assets/sounds/dish_ready.wav"),
    (SfxCue::Serve, "assets/sounds/serve.wav"),
    (SfxCue::Cash, "assets/sounds/cash.wav"),
    (SfxCue::LoungeSting, "assets/sounds/lounge_sting.wav"),
    (SfxCue::DayEnd, "assets/sounds/day_end.wav"),
    (SfxCue::Event, "assets/sounds/event.wav"),
];

impl AudioBank {
    pub async fn load(asset_pack: Option<&AssetPack>) -> Self {
        let mut sounds = HashMap::new();
        for (cue, path) in CLIPS {
            if let Ok(sound) = load_sound_from_pack_or_file(asset_pack, path).await {
                sounds.insert(cue, sound);
            }
        }
        Self {
            sounds,
            base_volume: 0.8,
            enabled: true,
        }
    }

    pub fn play(&self, cue: SfxCue) {
        if !self.enabled {
            return;
        }
        let Some(sound) = self.sounds.get(&cue) else {
            return;
        };
        let multiplier = match cue {
            // The sting is the loudest thing in the mix on purpose.
            SfxCue::LoungeSting => 1.0,
            SfxCue::DayEnd | SfxCue::Event => 0.8,
            _ => 0.6,
        };
        play_sound(
            sound,
            PlaySoundParams {
                looped: false,
                volume: self.base_volume * multiplier,
            },
        );
    }
}
