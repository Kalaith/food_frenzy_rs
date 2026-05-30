use crate::data::{GameData, STATION_COLORS};
use crate::gameplay::{
    attract_customer_type, craft_recipe, invite_customer_to_vip, serve_customer, try_prestige,
};
use crate::player::{
    clear_player_carry, interact_with_nearest_customer, interact_with_nearest_station,
    player_target_for_customer, select_station_with_player, send_player_to_customer,
    set_player_target, start_station_with_player,
};
use crate::state::{GameState, GuestState, ProgressionState};
use crate::ui::{SettingsAction, SettingsActions, TitleAction, TitleActions, UiActions};
use macroquad::prelude::*;

#[derive(Clone)]
pub enum UiCommand {
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

pub fn read_title_action(ui_hits: &TitleActions) -> Option<TitleAction> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    ui_hits.action_at(vec2(mouse_position().0, mouse_position().1))
}

pub fn read_settings_action(ui_hits: &SettingsActions) -> Option<SettingsAction> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    ui_hits.action_at(vec2(mouse_position().0, mouse_position().1))
}

pub fn read_input_action(ui_hits: UiActions) -> Option<UiCommand> {
    if !is_mouse_button_pressed(MouseButton::Left) {
        return None;
    }

    let click = vec2(mouse_position().0, mouse_position().1);
    if ui_hits
        .clear_selection
        .is_some_and(|rect| rect.contains(click))
    {
        return Some(UiCommand::ClearSelection);
    }

    find_vec_hit(ui_hits.station_select, click)
        .map(UiCommand::SelectDish)
        .or_else(|| find_vec_hit(ui_hits.station_cook, click).map(UiCommand::StartCooking))
        .or_else(|| find_map_hit(ui_hits.serve_customer, click).map(UiCommand::Serve))
        .or_else(|| find_map_hit(ui_hits.invite_customer, click).map(UiCommand::InviteVip))
        .or_else(|| find_map_hit(ui_hits.upgrade_buttons, click).map(UiCommand::BuyUpgrade))
        .or_else(|| find_map_hit(ui_hits.recipe_buttons, click).map(UiCommand::CraftRecipe))
        .or_else(|| find_map_hit(ui_hits.attract_buttons, click).map(UiCommand::AttractCustomer))
        .or_else(|| {
            ui_hits
                .prestige_button
                .filter(|rect| rect.contains(click))
                .map(|_| UiCommand::Prestige)
        })
}

pub fn apply_ui_command(
    command: UiCommand,
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    match command {
        UiCommand::StartCooking(station_color) => {
            start_station_with_player(
                &station_color,
                data,
                progression_state,
                selected_station,
                game_state,
            );
        }
        UiCommand::SelectDish(station_color) => {
            select_station_with_player(station_color, selected_station, game_state);
        }
        UiCommand::Serve(customer_id) => {
            serve_selected_customer(
                customer_id,
                data,
                selected_station,
                game_state,
                progression_state,
                guest_state,
            );
        }
        UiCommand::InviteVip(customer_id) => {
            invite_selected_customer(
                customer_id,
                data,
                selected_station,
                game_state,
                progression_state,
                guest_state,
            );
        }
        UiCommand::BuyUpgrade(upgrade_id) => {
            if progression_state.buy_upgrade(&upgrade_id) {
                game_state.add_message(format!("Upgrade purchased: {upgrade_id}"));
            } else {
                game_state.add_message(format!("Cannot purchase {upgrade_id} now."));
            }
        }
        UiCommand::CraftRecipe(recipe_id) => {
            craft_recipe(&recipe_id, data, game_state, progression_state);
        }
        UiCommand::AttractCustomer(customer_type_id) => {
            attract_customer_type(&customer_type_id, data, game_state, progression_state);
        }
        UiCommand::Prestige => {
            try_prestige(data, progression_state, game_state);
        }
        UiCommand::ClearSelection => {
            *selected_station = None;
            clear_player_carry(game_state);
        }
    }
}

pub fn handle_keyboard_shortcuts(
    data: &GameData,
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
        try_prestige(data, progression, game_state);
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

    for (key, station_color) in station_key_bindings() {
        if is_key_pressed(key) {
            start_station_with_player(
                station_color,
                data,
                progression,
                selected_station,
                game_state,
            );
        }
    }
}

pub fn clear_empty_selection(selected_station: &mut Option<String>, game_state: &mut GameState) {
    let should_clear = selected_station.as_ref().is_some_and(|station| {
        !game_state
            .cooking_stations
            .get(station)
            .is_some_and(|station| !station.dishes.is_empty())
    });
    if should_clear {
        *selected_station = None;
        if !game_state.player.clear_carry_on_arrival {
            clear_player_carry(game_state);
        }
    }
}

fn serve_selected_customer(
    customer_id: u32,
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    if let Some(station_color) = selected_station.clone() {
        if serve_customer(
            &station_color,
            customer_id,
            data,
            game_state,
            progression_state,
            guest_state,
        ) {
            send_player_to_customer(customer_id, &station_color, game_state);
            *selected_station = None;
        }
    } else {
        game_state.add_message("Select a cooked dish first.".to_string());
    }
}

fn invite_selected_customer(
    customer_id: u32,
    data: &GameData,
    selected_station: &mut Option<String>,
    game_state: &mut GameState,
    progression_state: &mut ProgressionState,
    guest_state: &mut GuestState,
) {
    let player_target = player_target_for_customer(customer_id, game_state);
    if invite_customer_to_vip(
        customer_id,
        data,
        game_state,
        progression_state,
        guest_state,
    ) {
        if let Some((x, y)) = player_target {
            set_player_target(game_state, x, y, "VIP", None, false);
        }
        *selected_station = None;
        clear_player_carry(game_state);
    }
}

fn find_vec_hit(values: Vec<(String, Rect)>, click: Vec2) -> Option<String> {
    values
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
        .map(|(value, _)| value)
}

fn find_map_hit<K>(values: std::collections::HashMap<K, Rect>, click: Vec2) -> Option<K> {
    values
        .into_iter()
        .find(|(_, rect)| rect.contains(click))
        .map(|(value, _)| value)
}

fn station_key_bindings() -> [(KeyCode, &'static str); 4] {
    [
        (KeyCode::Key1, STATION_COLORS[0]),
        (KeyCode::Key2, STATION_COLORS[1]),
        (KeyCode::Key3, STATION_COLORS[2]),
        (KeyCode::Key4, STATION_COLORS[3]),
    ]
}
