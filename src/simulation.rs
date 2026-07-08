mod guests;
mod spawning;
mod traits;

use crate::data::GameData;
use crate::engine::{max_customer_count, restaurant_table_position, CUSTOMER_WALK_SPEED};
use crate::gameplay::dish_display_name;
use crate::player::update_player_movement;
use crate::state::{GameState, GuestState, ProgressionState, Timers};

pub fn update_game_world(
    dt_ms: f32,
    data: &GameData,
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
    spawning::update_spawn(
        data,
        game_state,
        progression_state,
        guest_state,
        timers,
        now_ms,
    );
    update_customer_movement(dt_ms, data, progression_state, game_state);
    update_player_movement(dt_ms, game_state);
    guests::update_departures(dt_ms, data, game_state, progression_state, guest_state);
    guests::update_patience(data, game_state, progression_state, timers, now_ms);
    guests::update_satisfaction_decay(data, game_state, progression_state, timers);
    traits::update_traits(data, game_state, timers, progression_state);
    update_lounge(dt_ms, game_state);
}

fn update_customer_movement(
    dt_ms: f32,
    data: &GameData,
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

fn update_cooking(dt_ms: f32, data: &GameData, game_state: &mut GameState) {
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

fn update_lounge(dt_ms: f32, game_state: &mut GameState) {
    if !game_state.special_table_busy {
        return;
    }
    if game_state.special_table_timer > 0.0 {
        game_state.special_table_timer -= dt_ms;
    }
    if game_state.special_table_timer <= 0.0 {
        game_state.special_table_busy = false;
        game_state.special_table_timer = 0.0;
    }
}
