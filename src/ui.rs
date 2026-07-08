//! Macroquad UI drawing and hitbox collection for Feast Frenzy.

mod actors;
mod common;
mod dining;
mod floaters;
mod growth;
mod guest_status;
mod kitchen;
mod layout;
mod lounge;
mod menu;
mod sprites;
mod tutorial_panel;
mod types;

pub use layout::draw_and_collect_hitboxes;
pub use menu::{draw_settings_screen, draw_title_screen};
pub use types::{SettingsAction, SettingsActions, TitleAction, TitleActions, UiActions};
