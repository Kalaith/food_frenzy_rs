//! Sound effects, stored and played through `macroquad_toolkit::audio::SoundManager`.
//! Gameplay code queues `SfxCue`s on `GameState` (it has no audio access); the
//! app drains the queue each frame and plays through the bank. All clips live
//! in `assets/sounds/` and load from the asset pack with a loose-file fallback;
//! a missing clip degrades to silence rather than failing the boot.

use crate::state::SfxCue;
use macroquad_toolkit::assets::AssetPack;
use macroquad_toolkit::audio::SoundManager;

pub struct AudioBank {
    sounds: SoundManager<SfxCue>,
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
    pub async fn load(asset_pack: Option<AssetPack>) -> Self {
        let mut sounds = SoundManager::new();
        sounds.sfx_volume = 0.8;
        if let Some(pack) = asset_pack {
            sounds.add_asset_pack(pack);
        }
        for (cue, path) in CLIPS {
            let _ = sounds.load_sound(cue, path).await;
        }
        Self {
            sounds,
            enabled: true,
        }
    }

    pub fn play(&self, cue: SfxCue) {
        if !self.enabled {
            return;
        }
        let multiplier = match cue {
            // The sting is the loudest thing in the mix on purpose.
            SfxCue::LoungeSting => 1.0,
            SfxCue::DayEnd | SfxCue::Event => 0.8,
            _ => 0.6,
        };
        self.sounds.play_sfx(cue, multiplier);
    }
}
