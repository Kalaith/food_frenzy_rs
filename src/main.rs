//! Feast Frenzy migration to Macroquad.

mod data;
mod engine;
mod persistence;
mod state;
mod ui;

use crate::data::STATION_COLORS;
use crate::engine::*;
use crate::persistence::{load_game, save_game, FoodFrenzySave};
use crate::state::{GameState, GuestState, ProgressionState, Timers, INFINITE_INGREDIENTS};
use crate::ui::{
    draw_and_collect_hitboxes, draw_settings_screen, draw_title_screen, SettingsAction,
    SettingsActions, TitleAction, TitleActions, UiActions,
};
use macroquad::prelude::*;
use std::collections::{HashMap, HashSet};

const SAVE_INTERVAL_MS: f32 = 5_000.0;
const PLAYER_COOKING_START_LOCK_MS: f32 = 900.0;
const TITLE_TEXTURE_PATHS: [&str; 2] = [
    "assets/images/food_frenzy_title.png",
    "food_frenzy_title.png",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppScreen {
    Title,
    Settings,
    Playing,
}

#[derive(Clone)]
enum UiCommand {
    StartCooking(String),
    SelectDish(String),
    Serve(u32),
    InviteVip(u32),
    BuyUpgrade(String),
    CraftRecipe(String),
    AttractCustomer(String),
    Prestige,
    ClearSelection,
}

fn dish_display_name(data: &crate::data::GameData, color: &str) -> String {
    data.dish_type_by_color(color)
        .map(|dish| dish.name.clone())
        .unwrap_or_else(|| format!("{color} dish"))
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Feast Frenzy".to_owned(),
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        ..Default::default()
    }
}

async fn load_character_textures(data: &crate::data::GameData) -> HashMap<String, Texture2D> {
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

async fn load_title_texture() -> Option<Texture2D> {
    for path in TITLE_TEXTURE_PATHS {
        if let Ok(texture) = load_texture(path).await {
            texture.set_filter(FilterMode::Linear);
            return Some(texture);
        }
    }

    None
}

#[macroquad::main(window_conf)]
async fn main() {
    let data = data::GameData::load();
    let character_textures = load_character_textures(&data).await;
    let title_texture = load_title_texture().await;
    let mut game_state = GameState::new(&data);
    let mut progression_state = ProgressionState::from_game_data(&data);
    let mut guest_state = GuestState::new();
    let mut timers = Timers::new();
    let mut selected_station: Option<String> = None;
    let mut app_screen = AppScreen::Title;
    let mut title_message = String::new();
    let mut fullscreen_enabled = false;

    loop {
        let dt = get_frame_time();
        let dt_ms = dt * 1000.0;

        match app_screen {
            AppScreen::Title => {
                let title_hits = draw_title_screen(title_texture.as_ref(), &title_message);
                if let Some(action) = read_title_action(&title_hits) {
                    match action {
                        TitleAction::NewGame => {
                            start_new_game(
                                &data,
                                &mut game_state,
                                &mut progression_state,
                                &mut guest_state,
                                &mut timers,
                                &mut selected_station,
                            );
                            title_message.clear();
                            app_screen = AppScreen::Playing;
                        }
                        TitleAction::LoadGame => match load_saved_game(
                            &data,
                            &mut game_state,
                            &mut progression_state,
                            &mut guest_state,
                            &mut timers,
                            &mut selected_station,
                        ) {
                            Ok(()) => {
                                title_message.clear();
                                app_screen = AppScreen::Playing;
                            }
                            Err(message) => {
                                title_message = message;
                            }
                        },
                        TitleAction::Settings => {
                            app_screen = AppScreen::Settings;
                        }
                        TitleAction::Exit => {
                            macroquad::miniquad::window::quit();
                        }
                    }
                }

                if is_key_pressed(KeyCode::Enter) {
                    start_new_game(
                        &data,
                        &mut game_state,
                        &mut progression_state,
                        &mut guest_state,
                        &mut timers,
                        &mut selected_station,
                    );
                    title_message.clear();
                    app_screen = AppScreen::Playing;
                }

                next_frame().await;
                continue;
            }
            AppScreen::Settings => {
                let settings_hits = draw_settings_screen(fullscreen_enabled);
                if let Some(action) = read_settings_action(&settings_hits) {
                    match action {
                        SettingsAction::ToggleFullscreen => {
                            fullscreen_enabled = !fullscreen_enabled;
                            set_fullscreen(fullscreen_enabled);
                        }
                        SettingsAction::Back => {
                            app_screen = AppScreen::Title;
                        }
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    app_screen = AppScreen::Title;
                }

                next_frame().await;
                continue;
            }
            AppScreen::Playing => {}
        }

        update_game_world(
            dt_ms,
            &data,
            &mut game_state,
            &mut progression_state,
            &mut guest_state,
            &mut timers,
        );
        handle_player_keyboard_movement(dt_ms, &mut game_state);

        clear_background(Color::new(0.07, 0.07, 0.09, 1.0));
        let ui_hits = draw_and_collect_hitboxes(
            &game_state,
            &progression_state,
            &data,
            timers.elapsed_ms,
            &selected_station,
            &character_textures,
        );

        if let Some(command) = read_input_action(ui_hits) {
            match command {
                UiCommand::StartCooking(station_color) => {
                    start_station_with_player(
                        &station_color,
                        &data,
                        &progression_state,
                        &selected_station,
                        &mut game_state,
                    );
                }
                UiCommand::SelectDish(station_color) => {
                    select_station_with_player(
                        station_color,
                        &mut selected_station,
                        &mut game_state,
                    );
                }
                UiCommand::Serve(customer_id) => {
                    if let Some(station_color) = selected_station.clone() {
                        if serve_customer(
                            &station_color,
                            customer_id,
                            &data,
                            &mut game_state,
                            &mut progression_state,
                            &mut guest_state,
                        ) {
                            send_player_to_customer(customer_id, &station_color, &mut game_state);
                            selected_station = None;
                        }
                    } else {
                        game_state.add_message("Select a cooked dish first.".to_string());
                    }
                }
                UiCommand::InviteVip(customer_id) => {
                    let player_target = player_target_for_customer(customer_id, &game_state);
                    if invite_customer_to_vip(
                        customer_id,
                        &data,
                        &mut game_state,
                        &mut progression_state,
                        &mut guest_state,
                    ) {
                        if let Some((x, y)) = player_target {
                            set_player_target(&mut game_state, x, y, "VIP", None, false);
                        }
                        selected_station = None;
                        clear_player_carry(&mut game_state);
                    }
                }
                UiCommand::BuyUpgrade(upgrade_id) => {
                    if progression_state.buy_upgrade(&upgrade_id) {
                        game_state.add_message(format!("Upgrade purchased: {upgrade_id}"));
                    } else {
                        game_state.add_message(format!("Cannot purchase {upgrade_id} now."));
                    }
                }
                UiCommand::CraftRecipe(recipe_id) => {
                    craft_recipe(&recipe_id, &data, &mut game_state, &mut progression_state);
                }
                UiCommand::AttractCustomer(customer_type_id) => {
                    attract_customer_type(
                        &customer_type_id,
                        &data,
                        &mut game_state,
                        &mut progression_state,
                    );
                }
                UiCommand::Prestige => {
                    if progression_state.can_prestige(data.balance.prestige_score_requirement) {
                        progression_state.prestige(&data);
                        game_state.add_message(format!(
                            "Prestige complete! +{} currency gained.",
                            progression_state.prestige_level
                        ));
                    } else {
                        game_state.add_message("Prestige unavailable yet.".to_string());
                    }
                }
                UiCommand::ClearSelection => {
                    selected_station = None;
                    clear_player_carry(&mut game_state);
                }
            }
        }

        handle_keyboard_shortcuts(
            &data,
            &mut selected_station,
            &mut game_state,
            &mut progression_state,
            &mut guest_state,
        );

        if let Some(station) = &selected_station {
            if !game_state
                .cooking_stations
                .get(station)
                .is_some_and(|station| !station.dishes.is_empty())
            {
                selected_station = None;
                if !game_state.player.clear_carry_on_arrival {
                    clear_player_carry(&mut game_state);
                }
            }
        }

        timers.save_accum_ms += dt_ms;
        if timers.save_accum_ms >= SAVE_INTERVAL_MS {
            if let Err(error) = save_game(
                &game_state,
                &progression_state,
                &guest_state,
                &timers,
                &selected_station,
            ) {
                game_state.add_message(format!("Save failed: {error}"));
            }
            timers.save_accum_ms = 0.0;
        }

        next_frame().await;
    }
}

fn read_title_action(ui_hits: &TitleActions) -> Option<TitleAction> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    ui_hits.action_at(vec2(mouse_position().0, mouse_position().1))
}

fn read_settings_action(ui_hits: &SettingsActions) -> Option<SettingsAction> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    ui_hits.action_at(vec2(mouse_position().0, mouse_position().1))
}

fn start_new_game(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    selected_station: &mut Option<String>,
) {
    *game_state = GameState::new(data);
    *progression_state = ProgressionState::from_game_data(data);
    *guest_state = GuestState::new();
    *timers = Timers::new();
    *selected_station = None;
    initialize_active_game(
        data,
        progression_state,
        game_state,
        selected_station,
        timers,
        "New game started",
    );
}

fn load_saved_game(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    selected_station: &mut Option<String>,
) -> Result<(), String> {
    let Some(saved) = load_game().map_err(|error| format!("Load failed: {error}"))? else {
        return Err("No saved game found.".to_string());
    };

    restore_save(
        saved,
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        selected_station,
    );
    Ok(())
}

fn restore_save(
    saved: FoodFrenzySave,
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    selected_station: &mut Option<String>,
) {
    *game_state = saved.game_state;
    *progression_state = saved.progression_state;
    *guest_state = saved.guest_state;
    *timers = saved.timers;
    *selected_station = saved.selected_station;
    initialize_active_game(
        data,
        progression_state,
        game_state,
        selected_station,
        timers,
        "Loaded game save",
    );
}

fn initialize_active_game(
    data: &crate::data::GameData,
    progression: &mut ProgressionState,
    game_state: &mut GameState,
    selected_station: &mut Option<String>,
    timers: &mut Timers,
    startup_message: &str,
) {
    progression.ensure_customer_unlocks(data);
    ensure_compatibility(data, progression, game_state, selected_station);
    progression.set_upgrade_costs();
    if !startup_message.is_empty()
        && !game_state
            .messages
            .iter()
            .any(|message| message == startup_message)
    {
        game_state.add_message(startup_message.to_string());
    }
    if timers.next_spawn_ms <= 0.0 {
        timers.next_spawn_ms = f64::from(data.balance.customer_spawn_interval);
    }
    timers.save_accum_ms = 0.0;
}

fn read_input_action(ui_hits: UiActions) -> Option<UiCommand> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    let click = vec2(mouse_position().0, mouse_position().1);

    if let Some(rect) = &ui_hits.clear_selection {
        if rect.contains(click) {
            return Some(UiCommand::ClearSelection);
        }
    }

    if let Some((station, _rect)) = ui_hits
        .station_select
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::SelectDish(station));
    }

    if let Some((station, _rect)) = ui_hits
        .station_cook
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::StartCooking(station));
    }

    if let Some((customer_id, _)) = ui_hits
        .serve_customer
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::Serve(customer_id));
    }

    if let Some((customer_id, _)) = ui_hits
        .invite_customer
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::InviteVip(customer_id));
    }

    if let Some((upgrade, _)) = ui_hits
        .upgrade_buttons
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::BuyUpgrade(upgrade));
    }

    if let Some((recipe, _)) = ui_hits
        .recipe_buttons
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::CraftRecipe(recipe));
    }

    if let Some((customer_type_id, _)) = ui_hits
        .attract_buttons
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
    {
        return Some(UiCommand::AttractCustomer(customer_type_id));
    }

    if let Some(rect) = ui_hits.prestige_button {
        if rect.contains(click) {
            return Some(UiCommand::Prestige);
        }
    }

    None
}

fn ensure_compatibility(
    data: &crate::data::GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
    selected_station: &mut Option<String>,
) {
    for dish in &data.dish_types {
        if !game_state.cooking_stations.contains_key(&dish.color) {
            game_state.cooking_stations.insert(
                dish.color.clone(),
                crate::state::CookingStation::new(dish.color.clone()),
            );
        }
    }

    if !game_state
        .ingredients
        .contains_key(&data.balance.regular_ingredient_name)
    {
        game_state.ingredients.insert(
            data.balance.regular_ingredient_name.clone(),
            INFINITE_INGREDIENTS,
        );
    }

    if let Some(station) = selected_station {
        let exists = game_state.cooking_stations.contains_key(station);
        if !exists {
            *selected_station = None;
        }
    }

    let max_tables = max_customer_count(data, progression);
    for customer in &mut game_state.customers {
        let (target_x, target_y) = restaurant_table_position(customer.table_index, max_tables);
        customer.target_x = target_x;
        customer.target_y = target_y;
        if customer.floor_x == 0.0 && customer.floor_y == 0.0 {
            customer.floor_x = target_x;
            customer.floor_y = target_y;
            customer.is_seated = true;
        }
    }

    if game_state.player.carried_station.is_none()
        && (game_state.player.x + 165.0).abs() <= 1.0
        && (game_state.player.y - 72.0).abs() <= 1.0
    {
        let (x, y) = kitchen_pass_position();
        game_state.player.x = x;
        game_state.player.y = y;
        game_state.player.target_x = x;
        game_state.player.target_y = y;
    }
}

fn set_player_target(
    game_state: &mut GameState,
    x: f32,
    y: f32,
    task_label: &str,
    carried_station: Option<String>,
    clear_carry_on_arrival: bool,
) {
    game_state.player.target_x = x;
    game_state.player.target_y = y;
    game_state.player.task_label = task_label.to_string();
    game_state.player.carried_station = carried_station;
    game_state.player.clear_carry_on_arrival = clear_carry_on_arrival;
    game_state.player.lock_on_arrival_ms = 0.0;
}

fn send_player_to_station(
    station_color: &str,
    task_label: &str,
    carried_station: Option<String>,
    clear_carry_on_arrival: bool,
    game_state: &mut GameState,
) {
    let (x, y) = kitchen_station_position(station_color);
    set_player_target(
        game_state,
        x,
        y,
        task_label,
        carried_station,
        clear_carry_on_arrival,
    );
}

fn start_station_with_player(
    station_color: &str,
    data: &crate::data::GameData,
    progression: &ProgressionState,
    selected_station: &Option<String>,
    game_state: &mut GameState,
) -> bool {
    if game_state.player.action_lock_ms > 0.0 {
        game_state.add_message("Chef is finishing the cooking start.".to_string());
        return false;
    }

    let started = start_cooking(station_color, data, progression, game_state);
    if started {
        let carried_station = if game_state.player.clear_carry_on_arrival {
            None
        } else {
            selected_station.clone()
        };
        send_player_to_station(station_color, "Cooking", carried_station, false, game_state);
        game_state.player.lock_on_arrival_ms = PLAYER_COOKING_START_LOCK_MS;
        game_state.add_message(format!(
            "Started cooking {}.",
            dish_display_name(data, station_color)
        ));
    }
    started
}

fn select_station_with_player(
    station_color: String,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
) {
    if game_state.player.action_lock_ms > 0.0 {
        game_state.add_message("Chef is finishing the cooking start.".to_string());
        return;
    }

    if selected_station.as_deref() == Some(station_color.as_str()) {
        *selected_station = None;
        clear_player_carry(game_state);
    } else {
        *selected_station = Some(station_color.clone());
        send_player_to_station(
            &station_color,
            "Carrying",
            Some(station_color.clone()),
            false,
            game_state,
        );
    }
}

fn nearest_player_station(game_state: &GameState) -> Option<&'static str> {
    if game_state.player.x >= 0.0 {
        return None;
    }

    let mut best: Option<(&'static str, f32)> = None;
    for color in STATION_COLORS {
        let (x, y) = kitchen_station_position(color);
        let dx = game_state.player.x - x;
        let dy = game_state.player.y - y;
        let distance = (dx * dx + dy * dy).sqrt();
        let is_closer = match best {
            Some((_, best_distance)) => distance < best_distance,
            None => true,
        };
        if distance <= 74.0 && is_closer {
            best = Some((color, distance));
        }
    }

    best.map(|(color, _)| color)
}

fn interact_with_nearest_station(
    data: &crate::data::GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression: &ProgressionState,
) {
    let Some(station_color) = nearest_player_station(game_state) else {
        return;
    };

    let has_ready_dish = game_state
        .cooking_stations
        .get(station_color)
        .is_some_and(|station| !station.dishes.is_empty());
    if has_ready_dish {
        select_station_with_player(station_color.to_string(), selected_station, game_state);
    } else {
        start_station_with_player(
            station_color,
            data,
            progression,
            selected_station,
            game_state,
        );
    }
}

fn player_target_for_customer(customer_id: u32, game_state: &GameState) -> Option<(f32, f32)> {
    game_state
        .customers
        .iter()
        .find(|customer| customer.id == customer_id)
        .map(|customer| {
            (
                (customer.floor_x - 26.0).clamp(0.0, RESTAURANT_FLOOR_WIDTH),
                (customer.floor_y + 54.0).clamp(0.0, RESTAURANT_FLOOR_HEIGHT),
            )
        })
}

fn send_player_to_customer(customer_id: u32, station_color: &str, game_state: &mut GameState) {
    if let Some((x, y)) = player_target_for_customer(customer_id, game_state) {
        set_player_target(
            game_state,
            x,
            y,
            "Serving",
            Some(station_color.to_string()),
            true,
        );
    }
}

fn nearest_servable_customer(game_state: &GameState) -> Option<u32> {
    if game_state.player.x < 0.0 {
        return None;
    }

    let mut best: Option<(u32, f32)> = None;
    for customer in game_state
        .customers
        .iter()
        .filter(|customer| customer.is_seated)
    {
        let dx = game_state.player.x - customer.floor_x;
        let dy = game_state.player.y - customer.floor_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let is_closer = match best {
            Some((_, best_distance)) => distance < best_distance,
            None => true,
        };
        if distance <= 118.0 && is_closer {
            best = Some((customer.id, distance));
        }
    }

    best.map(|(customer_id, _)| customer_id)
}

fn interact_with_nearest_customer(
    data: &crate::data::GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    let Some(station_color) = selected_station.clone() else {
        return false;
    };
    let has_ready_dish = game_state
        .cooking_stations
        .get(&station_color)
        .is_some_and(|station| !station.dishes.is_empty());
    if !has_ready_dish {
        return false;
    }

    let Some(customer_id) = nearest_servable_customer(game_state) else {
        return false;
    };
    if serve_customer(
        &station_color,
        customer_id,
        data,
        game_state,
        progression,
        guest_state,
    ) {
        send_player_to_customer(customer_id, &station_color, game_state);
        *selected_station = None;
        true
    } else {
        false
    }
}

fn clear_player_carry(game_state: &mut GameState) {
    let (x, y) = kitchen_pass_position();
    game_state.player.target_x = x;
    game_state.player.target_y = y;
    game_state.player.carried_station = None;
    game_state.player.clear_carry_on_arrival = false;
    if game_state.player.task_label == "Carrying" || game_state.player.task_label == "Serving" {
        game_state.player.task_label = "Prep".to_string();
    }
}

fn handle_player_keyboard_movement(dt_ms: f32, game_state: &mut GameState) {
    if game_state.player.action_lock_ms > 0.0 {
        return;
    }

    let mut dx: f32 = 0.0;
    let mut dy: f32 = 0.0;

    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        dx -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        dx += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        dy -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        dy += 1.0;
    }

    if dx == 0.0 && dy == 0.0 {
        return;
    }

    let length: f32 = (dx * dx + dy * dy).sqrt();
    let travel = PLAYER_WALK_SPEED * (dt_ms / 1000.0);
    let player = &mut game_state.player;
    let mut next_x = player.x + dx / length * travel;
    let mut next_y = player.y + dy / length * travel;

    if next_x < 0.0 {
        next_x = next_x.clamp(KITCHEN_SERVICE_LEFT, -1.0);
        next_y = next_y.clamp(32.0, 160.0);
    } else {
        next_x = next_x.clamp(0.0, RESTAURANT_FLOOR_WIDTH);
        next_y = next_y.clamp(52.0, RESTAURANT_FLOOR_HEIGHT - 24.0);
    }

    player.x = next_x;
    player.y = next_y;
    player.target_x = next_x;
    player.target_y = next_y;
    player.lock_on_arrival_ms = 0.0;
    player.clear_carry_on_arrival = false;
    if player.carried_station.is_none() {
        player.task_label = if next_x < 0.0 { "Prep" } else { "Floor" }.to_string();
    }
}

fn handle_keyboard_shortcuts(
    data: &crate::data::GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    if is_key_pressed(KeyCode::C) {
        *selected_station = None;
        clear_player_carry(game_state);
        return;
    }
    if game_state.player.action_lock_ms > 0.0 {
        return;
    }
    if is_key_pressed(KeyCode::P) {
        if progression.can_prestige(data.balance.prestige_score_requirement) {
            progression.prestige(data);
            game_state.add_message(format!(
                "Prestige complete! +{} currency gained.",
                progression.prestige_level
            ));
        } else {
            game_state.add_message("Prestige unavailable yet.".to_string());
        }
    }
    if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Space) {
        if interact_with_nearest_customer(
            data,
            selected_station,
            game_state,
            progression,
            guest_state,
        ) {
            return;
        }
        interact_with_nearest_station(data, selected_station, game_state, progression);
    }

    if is_key_pressed(KeyCode::Key1) {
        start_station_with_player(
            STATION_COLORS[0],
            data,
            progression,
            selected_station,
            game_state,
        );
    }
    if is_key_pressed(KeyCode::Key2) {
        start_station_with_player(
            STATION_COLORS[1],
            data,
            progression,
            selected_station,
            game_state,
        );
    }
    if is_key_pressed(KeyCode::Key3) {
        start_station_with_player(
            STATION_COLORS[2],
            data,
            progression,
            selected_station,
            game_state,
        );
    }
    if is_key_pressed(KeyCode::Key4) {
        start_station_with_player(
            STATION_COLORS[3],
            data,
            progression,
            selected_station,
            game_state,
        );
    }
}

fn update_game_world(
    dt_ms: f32,
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
) {
    timers.elapsed_ms += f64::from(dt_ms);
    let now_ms = timers.elapsed_ms;
    timers.patience_accum_ms += dt_ms;
    timers.decay_accum_ms += dt_ms;
    timers.trait_accum_ms += dt_ms;
    update_cooking(dt_ms, data, game_state);
    update_spawn(
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        now_ms,
    );
    update_customer_movement(dt_ms, data, progression_state, game_state);
    update_player_movement(dt_ms, game_state);
    update_patience(
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        now_ms,
    );
    update_satisfaction_decay(data, game_state, progression_state, timers);
    update_traits(data, game_state, timers, progression_state);

    if game_state.special_table_busy {
        if game_state.special_table_timer > 0.0 {
            game_state.special_table_timer -= dt_ms;
        }
        if game_state.special_table_timer <= 0.0 {
            game_state.special_table_busy = false;
            game_state.special_table_timer = 0.0;
        }
    }
}

fn update_customer_movement(
    dt_ms: f32,
    data: &crate::data::GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
) {
    let max_tables = max_customer_count(data, progression);
    let travel = CUSTOMER_WALK_SPEED * (dt_ms / 1000.0);
    for customer in &mut game_state.customers {
        let (target_x, target_y) = restaurant_table_position(customer.table_index, max_tables);
        customer.target_x = target_x;
        customer.target_y = target_y;

        let dx = target_x - customer.floor_x;
        let dy = target_y - customer.floor_y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= travel || distance <= 1.0 {
            customer.floor_x = target_x;
            customer.floor_y = target_y;
            customer.is_seated = true;
        } else {
            let step = travel / distance;
            customer.floor_x += dx * step;
            customer.floor_y += dy * step;
            customer.is_seated = false;
        }
    }
}

fn update_player_movement(dt_ms: f32, game_state: &mut GameState) {
    let travel = PLAYER_WALK_SPEED * (dt_ms / 1000.0);
    let player = &mut game_state.player;
    if player.action_lock_ms > 0.0 {
        player.action_lock_ms = (player.action_lock_ms - dt_ms).max(0.0);
        if player.action_lock_ms <= 0.0 && player.task_label == "Cooking" {
            player.task_label = if player.x < 0.0 { "Prep" } else { "Floor" }.to_string();
        }
        return;
    }

    let dx = player.target_x - player.x;
    let dy = player.target_y - player.y;
    let distance = (dx * dx + dy * dy).sqrt();
    if distance <= travel || distance <= 1.0 {
        player.x = player.target_x;
        player.y = player.target_y;
        if player.lock_on_arrival_ms > 0.0 {
            player.action_lock_ms = player.lock_on_arrival_ms;
            player.lock_on_arrival_ms = 0.0;
            player.task_label = "Cooking".to_string();
            return;
        }
        if player.clear_carry_on_arrival {
            player.carried_station = None;
            player.clear_carry_on_arrival = false;
            player.task_label = "Floor".to_string();
        }
    } else if distance > 0.0 {
        let step = travel / distance;
        player.x += dx * step;
        player.y += dy * step;
    }
}

fn update_cooking(dt_ms: f32, data: &crate::data::GameData, game_state: &mut GameState) {
    let mut messages = Vec::new();
    for station in game_state.cooking_stations.values_mut() {
        if !station.is_cooking {
            continue;
        }

        if station.remaining_ms <= dt_ms {
            station.is_cooking = false;
            station.remaining_ms = 0.0;
            if let Some(dish) = data.dish_type_by_color(&station.color) {
                let cooked_name = crate::engine::random_dish_name(dish);
                station.dishes.push(cooked_name.clone());
                messages.push(format!(
                    "{} ready: {cooked_name}",
                    dish_display_name(data, &station.color)
                ));
            }
        } else {
            station.remaining_ms -= dt_ms;
        }
    }

    for message in messages {
        game_state.add_message(message);
    }
}

fn update_spawn(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    guest_state: &mut GuestState,
    timers: &mut Timers,
    now_ms: f64,
) {
    while timers.spawn_step < data.balance.initial_spawn_delays.len() {
        let initial_delay = data.balance.initial_spawn_delays[timers.spawn_step];
        if now_ms >= f64::from(initial_delay) {
            let _ = try_spawn_customer(data, game_state, progression, guest_state, now_ms);
            timers.spawn_step = timers.spawn_step.saturating_add(1);
        } else {
            break;
        }
    }

    let interval = f64::from(engine::spawn_interval_ms(data, progression));
    if now_ms >= timers.next_spawn_ms {
        let _ = try_spawn_customer(data, game_state, progression, guest_state, now_ms);
        timers.next_spawn_ms = now_ms + interval;
    }
}

fn update_patience(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    _guest_state: &mut GuestState,
    timers: &mut Timers,
    now_ms: f64,
) {
    if game_state.customers.is_empty() {
        timers.patience_accum_ms = 0.0;
        return;
    }

    while timers.patience_accum_ms >= 1_000.0_f32 {
        timers.patience_accum_ms -= 1_000.0;

        let patience_multiplier = patience_multiplier(progression);
        let mut removed_ids = Vec::new();
        for customer in &game_state.customers {
            let traits = customer.traits(data);
            let mut patience = data.balance.customer_patience_time * patience_multiplier;
            if traits.fast_spoilage {
                patience *= 0.55;
            }
            if now_ms - customer.arrived_at_ms > f64::from(patience) {
                removed_ids.push(customer.id);
            }
        }

        if removed_ids.is_empty() {
            continue;
        }

        let before = game_state.customers.len();
        game_state
            .customers
            .retain(|customer| !removed_ids.contains(&customer.id));
        let lost = before.saturating_sub(game_state.customers.len());
        if lost > 0 {
            game_state.combo = 0;
            for _ in 0..lost {
                progression.record_customer_lost();
            }
            game_state.add_message(format!("{lost} customers left from impatience."));
        }
    }
}

fn update_satisfaction_decay(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    timers: &mut Timers,
) {
    let decay_interval = data.balance.satisfaction_decay_interval;
    if decay_interval <= 0.0 {
        return;
    }
    if game_state.customers.is_empty() {
        timers.decay_accum_ms = 0.0;
        return;
    }

    let decay_rate = satisfaction_decay_rate(data, progression);
    let decay_tick = decay_interval as f32;
    while timers.decay_accum_ms >= decay_tick {
        timers.decay_accum_ms -= decay_tick;
        for customer in &mut game_state.customers {
            if let Some(current) = customer.satisfaction.get_mut("blue") {
                *current = (*current - decay_rate).max(0.0);
            }
            if let Some(current) = customer.satisfaction.get_mut("green") {
                *current = (*current - decay_rate).max(0.0);
            }
            if let Some(current) = customer.satisfaction.get_mut("yellow") {
                *current = (*current - decay_rate).max(0.0);
            }
            if let Some(current) = customer.satisfaction.get_mut("red") {
                *current = (*current - decay_rate).max(0.0);
            }
            customer.refresh_totals();
        }
    }
}

fn update_traits(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    timers: &mut Timers,
    progression: &ProgressionState,
) {
    let tick = data.balance.trait_tick_interval;
    if tick <= 0.0 || game_state.customers.is_empty() {
        timers.trait_accum_ms = 0.0;
        return;
    }

    let max_tables = max_customer_count(data, progression);
    let mut occupied = HashSet::new();
    for customer in &game_state.customers {
        if customer.table_index < max_tables {
            occupied.insert(customer.table_index);
        }
    }

    let trait_tick = tick as f32;
    while timers.trait_accum_ms >= trait_tick {
        timers.trait_accum_ms -= trait_tick;

        let customer_len = game_state.customers.len();
        for index in 0..customer_len {
            let traits = game_state.customers[index].traits(data);

            if traits.can_wander {
                if chance(CAN_WANDER_CHANCE) && max_tables > 0 {
                    let empty_tables: Vec<usize> = (0..max_tables)
                        .filter(|idx| !occupied.contains(idx))
                        .collect();
                    if !empty_tables.is_empty() {
                        if let Some(next) =
                            random_index(empty_tables.len()).map(|idx| empty_tables[idx])
                        {
                            if let Some(previous) =
                                game_state.customers.get(index).map(|c| c.table_index)
                            {
                                if previous < max_tables {
                                    occupied.remove(&previous);
                                    occupied.insert(next);
                                    let display_name = if let Some(customer) =
                                        game_state.customers.get_mut(index)
                                    {
                                        customer.table_index = next;
                                        customer.is_seated = false;
                                        Some(customer.display_name.clone())
                                    } else {
                                        None
                                    };
                                    if let Some(display_name) = display_name {
                                        game_state.add_message(format!(
                                            "{display_name} moved to table {}",
                                            next + 1
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if traits.can_steal_food && chance(FOX_STEAL_CHANCE) {
                if let Some((station_color, dish_name)) = steal_or_throw_dish(game_state) {
                    let station_name = dish_display_name(data, &station_color);
                    game_state.add_message(format!(
                        "{} stole {dish_name} from {station_name}",
                        game_state.customers[index].display_name
                    ));
                }
            }

            if traits.throws_food && chance(MONKEY_THROW_CHANCE) {
                if game_state.customers[index].total_satisfaction < 60.0
                    && game_state.customers[index].total_satisfaction > 0.0
                {
                    if let Some((station_color, dish_name)) = steal_or_throw_dish(game_state) {
                        let display_name = &game_state.customers[index].display_name;
                        let station_name = dish_display_name(data, &station_color);
                        game_state.add_message(format!(
                            "{display_name} threw away {dish_name} from {station_name}"
                        ));
                        game_state.combo = 0;
                    }
                }
            }

            if traits.fast_spoilage {
                if let Some(current) = game_state.customers[index].satisfaction.get_mut("blue") {
                    *current = (*current - 2.0).max(0.0);
                }
                if let Some(current) = game_state.customers[index].satisfaction.get_mut("green") {
                    *current = (*current - 2.0).max(0.0);
                }
                if let Some(current) = game_state.customers[index].satisfaction.get_mut("yellow") {
                    *current = (*current - 2.0).max(0.0);
                }
                if let Some(current) = game_state.customers[index].satisfaction.get_mut("red") {
                    *current = (*current - 2.0).max(0.0);
                }
                game_state.customers[index].refresh_totals();
            }
        }
    }
}

fn try_spawn_customer(
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &ProgressionState,
    guest_state: &mut GuestState,
    now_ms: f64,
) -> bool {
    let max_customers = max_customer_count(data, progression);
    if game_state.customers.len() >= max_customers {
        return false;
    }

    let active_guest_ids: Vec<String> = game_state
        .customers
        .iter()
        .map(|customer| customer.guest_id.clone())
        .collect();
    let returning = if chance(RETURNING_GUEST_CHANCE) {
        guest_state
            .get_returning_unlocked_guest(&progression.unlocked_customer_types, &active_guest_ids)
    } else {
        None
    };

    let selected_type_id = returning
        .as_ref()
        .and_then(|guest| data.customer_type_by_id(&guest.customer_type))
        .cloned()
        .or_else(|| {
            let unlocked_customer_types: Vec<_> = data
                .customer_types
                .iter()
                .filter(|customer_type| progression.is_customer_unlocked(&customer_type.id))
                .collect();
            random_index(unlocked_customer_types.len())
                .map(|index| unlocked_customer_types[index].clone())
        });
    let Some(customer_type) = selected_type_id else {
        return false;
    };

    let guest_record = if let Some(guest) = returning {
        guest
    } else {
        let guest_name = random_guest_name();
        guest_state.create_guest(&guest_name, &customer_type.id)
    };

    let table_index = first_empty_table(game_state, max_customers);
    if table_index.is_none() {
        return false;
    }
    let table_index = table_index.unwrap_or_default();

    let traits = customer_type.special_traits.clone().unwrap_or_default();
    let max_satisfaction =
        max_satisfaction_for_customer(&data.balance, &traits, progression.feeding_capacity_bonus);
    let customer_id = game_state.next_customer_id();
    let (floor_x, floor_y) = restaurant_entrance_position();
    let (target_x, target_y) = restaurant_table_position(table_index, max_customers);
    let customer = crate::state::Customer {
        id: customer_id,
        guest_id: guest_record.id.clone(),
        display_name: guest_record.name.clone(),
        customer_type: customer_type.id.clone(),
        satisfaction: crate::state::Satisfaction::default(),
        max_satisfaction,
        deliciousness: customer_type.base_deliciousness,
        total_satisfaction: 0.0,
        overfed: false,
        table_index,
        arrived_at_ms: now_ms,
        floor_x,
        floor_y,
        target_x,
        target_y,
        is_seated: false,
    };

    game_state.customers.push(customer);
    guest_state.record_guest_visit(&guest_record.id);
    game_state.add_message(format!(
        "{} arrived at table {}.",
        guest_record.name,
        table_index + 1
    ));

    true
}

fn first_empty_table(game_state: &GameState, max_customers: usize) -> Option<usize> {
    (0..max_customers).find(|index| {
        game_state
            .customers
            .iter()
            .all(|customer| customer.table_index != *index)
    })
}

fn start_cooking(
    station_color: &str,
    data: &crate::data::GameData,
    progression: &ProgressionState,
    game_state: &mut GameState,
) -> bool {
    let cooking_slot_limit = data.balance.cooking_slots_limit;
    let Some(station) = game_state.station_mut(station_color) else {
        return false;
    };

    if station.can_cook(cooking_slot_limit) {
        station.is_cooking = true;
        station.remaining_ms = cooking_time_ms(data, progression, station_color);
        true
    } else {
        false
    }
}

fn serve_customer(
    station_color: &str,
    customer_id: u32,
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    let customer_pos = game_state
        .customers
        .iter()
        .position(|customer| customer.id == customer_id);
    let Some(pos) = customer_pos else {
        return false;
    };
    if !game_state.customers[pos].is_seated {
        game_state.add_message(format!(
            "{} is still walking to their table.",
            game_state.customers[pos].display_name
        ));
        return false;
    }

    let dish_name = {
        let Some(station) = game_state.station_mut(station_color) else {
            return false;
        };
        if station.dishes.is_empty() {
            return false;
        }
        station.dishes.remove(0)
    };

    let (satisfaction_gain, delicious_gain, preferred) = {
        let customer = &game_state.customers[pos];
        let traits = customer.traits(data);
        if data.customer_type_by_id(&customer.customer_type).is_some() {
            serving_gain(data, &customer.customer_type, &traits, station_color)
        } else {
            (data.balance.base_satisfaction_gain, 0.0, false)
        }
    };

    {
        let customer = &mut game_state.customers[pos];
        let traits = customer.traits(data);
        if let Some(existing) = customer.satisfaction.get_mut(station_color) {
            let max_total = customer.max_satisfaction.get(station_color).unwrap_or(40.0);
            let limit = max_total * overfeed_multiplier(data, &traits);
            *existing = (*existing + satisfaction_gain).min(limit);
        }
        customer.deliciousness = (customer.deliciousness + delicious_gain).min(5.0);
        customer.refresh_totals();
    }

    let is_overfed = {
        let customer = &game_state.customers[pos];
        customer.overfed
    };
    let score_gain = serving_points(data, satisfaction_gain, preferred) as f64;
    let total_gain = add_score(game_state, progression, score_gain, true);
    progression.record_served_dish(preferred, is_overfed);
    guest_state.record_guest_fed(&game_state.customers[pos].guest_id);
    game_state.combo = game_state.combo.saturating_add(1);
    game_state.chain = game_state.chain.saturating_add(1);
    game_state.add_message(format!(
        "{} served {dish_name} for {} satisfaction (+{total_gain} points)",
        game_state.customers[pos].display_name, satisfaction_gain
    ));

    let can_vip = can_process_customer(&game_state.customers[pos], data);
    if can_vip {
        game_state.add_message(format!(
            "{} is ready for the Last Meal Lounge.",
            game_state.customers[pos].display_name
        ));
    }

    true
}

fn invite_customer_to_vip(
    customer_id: u32,
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    guest_state: &mut GuestState,
) -> bool {
    if game_state.special_table_busy {
        game_state.add_message("Last Meal Lounge is occupied right now.".to_string());
        return false;
    }

    let position = game_state
        .customers
        .iter()
        .position(|customer| customer.id == customer_id);
    let Some(index) = position else {
        return false;
    };
    if !game_state.customers[index].is_seated {
        game_state.add_message(format!(
            "{} needs to sit before lounge service.",
            game_state.customers[index].display_name
        ));
        return false;
    }
    if !can_process_customer(&game_state.customers[index], data) {
        game_state.add_message("This customer is not ready for VIP yet.".to_string());
        return false;
    }

    if !chance(VIP_ACCEPT_CHANCE) {
        game_state.add_message(format!(
            "{} declined the invitation.",
            game_state.customers[index].display_name
        ));
        return false;
    }

    let customer = game_state.customers.remove(index);
    let meat_gain = vip_meat_gain(&customer, data, progression);
    let meat_type = format!("{}-meat", customer.customer_type);
    let entry = game_state.ingredients.entry(meat_type.clone()).or_insert(0);
    if *entry != INFINITE_INGREDIENTS {
        *entry = entry.saturating_add(meat_gain);
    }

    let chain_value = game_state.chain.saturating_add(1);
    game_state.chain = chain_value;
    game_state.combo = game_state.combo.saturating_add(1);
    guest_state.record_guest_processed(&customer.guest_id);

    let meal_points = ((vip_points(&customer, data) as f64) + (meat_gain as f64 * 10.0))
        * data.balance.base_score_multiplier;
    let awarded = add_score(game_state, progression, meal_points, true);
    progression.add_currency((awarded / 5).max(0));
    progression.record_processed_customer(&customer.customer_type, chain_value);
    game_state.special_table_busy = true;
    game_state.special_table_timer = data.balance.special_table_process_time;
    game_state.add_message(format!(
        "{} entered the Last Meal Lounge. {} gained with +{} {}.",
        customer.display_name, awarded, meat_gain, meat_type
    ));

    true
}

fn attract_customer_type(
    customer_type_id: &str,
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let Some(customer_type) = data.customer_type_by_id(customer_type_id) else {
        game_state.add_message("Unknown clientele.".to_string());
        return;
    };
    if progression.is_customer_unlocked(customer_type_id) {
        game_state.add_message(format!("{} already visits the cafe.", customer_type.name));
        return;
    }

    let missing: Vec<String> = customer_type
        .unlock_cost
        .iter()
        .filter_map(|(ingredient, amount)| {
            if game_state.has_ing(ingredient, *amount) {
                None
            } else {
                let current = game_state.ingredients.get(ingredient).copied().unwrap_or(0);
                Some(format!("{ingredient} {current}/{amount}"))
            }
        })
        .collect();
    if !missing.is_empty() {
        game_state.add_message(format!(
            "Need more ingredients to attract {}: {}.",
            customer_type.name,
            missing.join(", ")
        ));
        return;
    }

    for (ingredient, amount) in &customer_type.unlock_cost {
        if !game_state.remove_ingredients(ingredient, *amount) {
            game_state.add_message(format!("Missing {ingredient} while attracting clientele."));
            return;
        }
    }

    if progression.unlock_customer_type(customer_type_id) {
        game_state.add_message(format!(
            "{} clientele unlocked. They can now walk in.",
            customer_type.name
        ));
    }
}

fn craft_recipe(
    recipe_id: &str,
    data: &crate::data::GameData,
    game_state: &mut GameState,
    progression: &mut ProgressionState,
) {
    let recipe_pos = progression
        .recipes
        .iter()
        .position(|recipe| recipe.id == recipe_id);
    let Some(index) = recipe_pos else {
        game_state.add_message("Unknown recipe.".to_string());
        return;
    };

    let recipe = progression.recipes[index].clone();
    if !recipe.unlocked {
        game_state.add_message(format!("{} is locked.", recipe.name));
        return;
    }

    let has_ingredients = recipe.ingredients.iter().all(|(ingredient, amount)| {
        if ingredient == &data.balance.regular_ingredient_name {
            true
        } else {
            game_state
                .ingredients
                .get(ingredient)
                .is_some_and(|current| *current >= *amount)
        }
    });

    if !has_ingredients {
        game_state.add_message(format!("Not enough ingredients for {}.", recipe.name));
        return;
    }

    for (ingredient, amount) in &recipe.ingredients {
        if ingredient == &data.balance.regular_ingredient_name {
            continue;
        }
        let removed = game_state.remove_ingredients(ingredient, *amount);
        if !removed {
            game_state.add_message(format!("Missing {ingredient} while crafting."));
            return;
        }
    }

    let points = recipe.base_value as f64
        * recipe.profit_multiplier
        * recipe_value_multiplier(progression)
        * data.balance.base_score_multiplier;
    let awarded = add_score(game_state, progression, points, false);
    progression.add_currency(awarded / 4);
    let bonus_capacity = recipe_capacity_gain(progression, recipe.capacity_bonus);
    progression.record_crafted_recipe(&recipe.id, bonus_capacity);
    game_state.add_message(format!("Crafted {} for {} points.", recipe.name, awarded));
}

fn add_score(
    game_state: &mut GameState,
    progression: &mut ProgressionState,
    points: f64,
    apply_combo: bool,
) -> i64 {
    let combo_multiplier = if apply_combo {
        let combo_boost = progression.get_effect("combo_multiplier", 1.0);
        1.0 + (f64::from(game_state.combo) * 0.1 * combo_boost)
    } else {
        1.0
    };
    let prestige_multiplier = 1.0 + (f64::from(progression.prestige_points) * 0.03);
    let awarded = (points * combo_multiplier * prestige_multiplier).max(0.0);

    let next_score = awarded.floor() as i64;
    if next_score > 0 {
        progression.record_score(next_score);
        game_state.score = game_state.score.saturating_add(next_score);
    }

    next_score
}

fn random_guest_name() -> String {
    const NAMES: &[&str] = &[
        "Marnie", "Lark", "Penny", "Bram", "Sora", "Rin", "Nova", "Haze", "Felix", "Tara", "Lune",
        "Milo", "Ari", "Violet",
    ];
    random_index(NAMES.len())
        .map(|index| NAMES[index])
        .unwrap_or("Guest")
        .to_string()
}

fn steal_or_throw_dish(game_state: &mut GameState) -> Option<(String, String)> {
    let candidates: Vec<String> = game_state
        .cooking_stations
        .iter()
        .filter_map(|(color, station)| {
            if station.dishes.is_empty() {
                None
            } else {
                Some(color.clone())
            }
        })
        .collect();

    let Some(station_color) = random_index(candidates.len()).map(|index| candidates[index].clone())
    else {
        return None;
    };

    let station = game_state.cooking_stations.get_mut(&station_color)?;
    if station.dishes.is_empty() {
        return None;
    }
    let dish = station.dishes.remove(0);
    Some((station_color, dish))
}
