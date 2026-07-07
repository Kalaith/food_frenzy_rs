use crate::assets::{load_asset_pack, load_character_textures, load_title_texture};
use crate::commands::{
    apply_ui_command, clear_empty_selection, handle_keyboard_shortcuts, read_input_action,
    read_settings_action, read_title_action,
};
use crate::data::GameData;
use crate::lifecycle::{load_saved_game, start_new_game};
use crate::persistence::save_game;
use crate::player::handle_player_keyboard_movement;
use crate::simulation::update_game_world;
use crate::state::{GameState, GuestState, ProgressionState, Timers};
use crate::ui::{
    draw_and_collect_hitboxes, draw_settings_screen, draw_title_screen, SettingsAction, TitleAction,
};
use macroquad::prelude::*;
use macroquad_toolkit::capture;
use std::collections::HashMap;

const SAVE_INTERVAL_MS: f32 = 5_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppScreen {
    Title,
    Settings,
    Playing,
}

struct App {
    data: GameData,
    character_textures: HashMap<String, Texture2D>,
    title_texture: Option<Texture2D>,
    game_state: GameState,
    progression_state: ProgressionState,
    guest_state: GuestState,
    timers: Timers,
    selected_station: Option<String>,
    app_screen: AppScreen,
    title_message: String,
    fullscreen_enabled: bool,
}

pub async fn run() {
    let mut app = App::load().await;

    // Screenshot harness: when FOOD_FRENZY_CAPTURE_PATH is set, seed a scene,
    // simulate deterministic frames, write a PNG, and exit. App::tick() has no
    // dt parameter (it reads get_frame_time() internally), so the capture
    // closure ignores the fixed timestep the harness would otherwise pass.
    if let Some(config) = capture::CaptureConfig::from_env("FOOD_FRENZY") {
        app.begin_capture_scene(&config.scene);
        capture::run_capture(&config, |_dt| {
            app.tick();
        })
        .await;
        return;
    }

    loop {
        app.tick();
        next_frame().await;
    }
}

impl App {
    async fn load() -> Self {
        let data = GameData::load();
        let asset_pack = load_asset_pack().await;
        let character_textures = load_character_textures(&data, asset_pack.as_ref()).await;
        let title_texture = load_title_texture(asset_pack.as_ref()).await;
        let game_state = GameState::new(&data);
        let progression_state = ProgressionState::from_game_data(&data);

        Self {
            data,
            character_textures,
            title_texture,
            game_state,
            progression_state,
            guest_state: GuestState::new(),
            timers: Timers::new(),
            selected_station: None,
            app_screen: AppScreen::Title,
            title_message: String::new(),
            fullscreen_enabled: false,
        }
    }

    /// Seed a specific scene for the screenshot harness.
    fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "title" => self.app_screen = AppScreen::Title,
            "settings" => self.app_screen = AppScreen::Settings,
            _ => {
                // Default: jump straight into gameplay on a fresh save, then
                // warm the world into a lively state so headless captures show
                // seated guests and active stations instead of an empty room.
                self.start_new_game();
                self.seed_gameplay_demo();
            }
        }
    }

    /// Advance a fresh game into a representative mid-service moment for the
    /// screenshot harness: guests seated, one dish plated, another cooking.
    /// Capture-only — the normal game loop never calls this.
    fn seed_gameplay_demo(&mut self) {
        let step = |app: &mut Self, dt_ms: f32| {
            update_game_world(
                dt_ms,
                &app.data,
                &mut app.game_state,
                &mut app.progression_state,
                &mut app.guest_state,
                &mut app.timers,
            );
        };

        // Run ~12s of simulation so both opening guests arrive and take a table.
        for _ in 0..60 {
            step(self, 200.0);
        }
        // Plate a quick appetizer and get a slower roast going for visible state.
        crate::gameplay::start_cooking(
            "blue",
            &self.data,
            &self.progression_state,
            &mut self.game_state,
        );
        crate::gameplay::start_cooking(
            "yellow",
            &self.data,
            &self.progression_state,
            &mut self.game_state,
        );
        for _ in 0..18 {
            step(self, 200.0);
        }
        // Carry the ready appetizer so the serve prompt and buttons are shown.
        self.selected_station = Some("blue".to_string());
    }

    fn tick(&mut self) {
        let dt_ms = get_frame_time() * 1000.0;
        match self.app_screen {
            AppScreen::Title => self.tick_title(),
            AppScreen::Settings => self.tick_settings(),
            AppScreen::Playing => self.tick_playing(dt_ms),
        }
    }

    fn tick_title(&mut self) {
        let title_hits = draw_title_screen(self.title_texture.as_ref(), &self.title_message);
        if let Some(action) = read_title_action(&title_hits) {
            self.handle_title_action(action);
        }

        if is_key_pressed(KeyCode::Enter) {
            self.start_new_game();
        }
    }

    fn tick_settings(&mut self) {
        let settings_hits = draw_settings_screen(self.fullscreen_enabled);
        if let Some(action) = read_settings_action(&settings_hits) {
            match action {
                SettingsAction::ToggleFullscreen => {
                    self.fullscreen_enabled = !self.fullscreen_enabled;
                    set_fullscreen(self.fullscreen_enabled);
                }
                SettingsAction::Back => {
                    self.app_screen = AppScreen::Title;
                }
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            self.app_screen = AppScreen::Title;
        }
    }

    fn tick_playing(&mut self, dt_ms: f32) {
        update_game_world(
            dt_ms,
            &self.data,
            &mut self.game_state,
            &mut self.progression_state,
            &mut self.guest_state,
            &mut self.timers,
        );
        handle_player_keyboard_movement(dt_ms, &mut self.game_state);

        let ui_hits = draw_and_collect_hitboxes(
            &self.game_state,
            &self.progression_state,
            &self.data,
            self.timers.elapsed_ms,
            &self.selected_station,
            &self.character_textures,
        );

        if let Some(command) = read_input_action(ui_hits) {
            apply_ui_command(
                command,
                &self.data,
                &mut self.selected_station,
                &mut self.game_state,
                &mut self.progression_state,
                &mut self.guest_state,
            );
        }

        handle_keyboard_shortcuts(
            &self.data,
            &mut self.selected_station,
            &mut self.game_state,
            &mut self.progression_state,
            &mut self.guest_state,
        );
        clear_empty_selection(&mut self.selected_station, &mut self.game_state);
        self.save_if_due(dt_ms);
    }

    fn handle_title_action(&mut self, action: TitleAction) {
        match action {
            TitleAction::NewGame => self.start_new_game(),
            TitleAction::LoadGame => self.load_saved_game(),
            TitleAction::Settings => {
                self.app_screen = AppScreen::Settings;
            }
            TitleAction::Exit => {
                macroquad::miniquad::window::quit();
            }
        }
    }

    fn start_new_game(&mut self) {
        start_new_game(
            &self.data,
            &mut self.game_state,
            &mut self.progression_state,
            &mut self.guest_state,
            &mut self.timers,
            &mut self.selected_station,
        );
        self.title_message.clear();
        self.app_screen = AppScreen::Playing;
    }

    fn load_saved_game(&mut self) {
        match load_saved_game(
            &self.data,
            &mut self.game_state,
            &mut self.progression_state,
            &mut self.guest_state,
            &mut self.timers,
            &mut self.selected_station,
        ) {
            Ok(()) => {
                self.title_message.clear();
                self.app_screen = AppScreen::Playing;
            }
            Err(message) => {
                self.title_message = message;
            }
        }
    }

    fn save_if_due(&mut self, dt_ms: f32) {
        self.timers.save_accum_ms += dt_ms;
        if self.timers.save_accum_ms < SAVE_INTERVAL_MS {
            return;
        }

        if let Err(error) = save_game(
            &self.game_state,
            &self.progression_state,
            &self.guest_state,
            &self.timers,
            &self.selected_station,
        ) {
            self.game_state.add_message(format!("Save failed: {error}"));
        }
        self.timers.save_accum_ms = 0.0;
    }
}
