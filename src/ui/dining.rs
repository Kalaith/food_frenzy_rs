use super::actors::{draw_customer_sprite, draw_player_actor};
use super::common::{
    dish_label, draw_bar, draw_button, ellipsize, floor_to_screen, patience_color,
    patience_remaining_ratio, station_draw_color, LINE, MUTED, TEXT,
};
use super::types::UiActions;
use crate::data::GameData;
use crate::engine::{max_customer_count, restaurant_entrance_position, restaurant_table_position};
use crate::state::{Customer, GameState, ProgressionState};
use macroquad::prelude::*;
use std::collections::HashMap;

fn draw_floor_pattern(floor: Rect) {
    draw_rectangle(
        floor.x,
        floor.y,
        floor.w,
        floor.h,
        Color::new(0.105, 0.095, 0.085, 1.0),
    );
    let tile = 68.0;
    let mut x = floor.x;
    while x < floor.x + floor.w {
        draw_line(
            x,
            floor.y,
            x,
            floor.y + floor.h,
            1.0,
            Color::new(0.15, 0.14, 0.13, 1.0),
        );
        x += tile;
    }
    let mut y = floor.y;
    while y < floor.y + floor.h {
        draw_line(
            floor.x,
            y,
            floor.x + floor.w,
            y,
            1.0,
            Color::new(0.15, 0.14, 0.13, 1.0),
        );
        y += tile;
    }
}

fn draw_table(
    center: Vec2,
    table_index: usize,
    customer: Option<&Customer>,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
    selected_station: &Option<String>,
    game: &GameState,
    ui: &mut UiActions,
) {
    let table_w = 154.0;
    let table_h = 90.0;
    let occupied = customer.is_some();
    let ready_for_lounge =
        customer.is_some_and(|customer| crate::engine::can_process_customer(customer, data));
    let table_color = if occupied {
        Color::new(0.32, 0.22, 0.14, 1.0)
    } else {
        Color::new(0.20, 0.16, 0.13, 1.0)
    };
    let outline = if ready_for_lounge {
        SKYBLUE
    } else if occupied {
        Color::new(0.70, 0.50, 0.30, 1.0)
    } else {
        Color::new(0.42, 0.34, 0.25, 1.0)
    };
    let rect = Rect::new(
        center.x - table_w * 0.5,
        center.y - table_h * 0.5,
        table_w,
        table_h,
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, table_color);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if ready_for_lounge { 2.5 } else { 1.5 },
        outline,
    );
    draw_text(
        &format!("T{}", table_index + 1),
        rect.x + 10.0,
        rect.y + 22.0,
        17.0,
        Color::new(0.78, 0.67, 0.52, 1.0),
    );

    let Some(customer) = customer else {
        draw_text("open", rect.x + 48.0, rect.y + 22.0, 14.0, MUTED);
        return;
    };

    let patience = patience_remaining_ratio(customer, data, progression, now_ms);
    draw_text(
        if customer.is_seated {
            "seated"
        } else {
            "arriving"
        },
        rect.x + 48.0,
        rect.y + 22.0,
        14.0,
        if customer.is_seated { LIGHTGRAY } else { MUTED },
    );
    draw_text(
        &ellipsize(&customer.display_name, 14),
        rect.x + 10.0,
        rect.y + 46.0,
        15.0,
        TEXT,
    );

    let mut chip_x = rect.x + 10.0;
    if let Some(customer_type) = data.customer_type_by_id(&customer.customer_type) {
        for dish_color in customer_type.preferred_dishes.iter().take(3) {
            draw_circle(
                chip_x + 6.0,
                rect.y + 62.0,
                5.0,
                station_draw_color(dish_color),
            );
            chip_x += 18.0;
        }
    }
    draw_text("order", chip_x + 2.0, rect.y + 66.0, 12.0, MUTED);
    draw_bar(
        rect.x + 10.0,
        rect.y + rect.h - 13.0,
        rect.w - 20.0,
        6.0,
        patience,
        1.0,
        patience_color(patience),
    );

    let can_serve = selected_station
        .as_ref()
        .and_then(|station| game.cooking_stations.get(station))
        .is_some_and(|station| !station.dishes.is_empty())
        && customer.is_seated;
    if can_serve {
        let serve_rect = Rect::new(rect.x + rect.w - 62.0, rect.y + 12.0, 50.0, 24.0);
        draw_button(serve_rect, "Serve", true, false);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, SKYBLUE);
        ui.serve_customer.insert(customer.id, serve_rect);
    }
}

fn draw_last_meal_lounge(floor: Rect, game: &GameState, data: &GameData) {
    let lounge = Rect::new(
        floor.x + floor.w - 244.0,
        floor.y + floor.h - 128.0,
        216.0,
        96.0,
    );
    let ready_guest = game
        .customers
        .iter()
        .find(|customer| customer.is_seated && crate::engine::can_process_customer(customer, data));
    let is_active = game.special_table_busy || ready_guest.is_some();
    draw_rectangle(
        lounge.x,
        lounge.y,
        lounge.w,
        lounge.h,
        if is_active {
            Color::new(0.12, 0.14, 0.16, 1.0)
        } else {
            Color::new(0.075, 0.075, 0.085, 1.0)
        },
    );
    draw_rectangle_lines(
        lounge.x,
        lounge.y,
        lounge.w,
        lounge.h,
        if is_active { 2.0 } else { 1.0 },
        if is_active { SKYBLUE } else { LINE },
    );
    draw_text(
        "Last Meal Lounge",
        lounge.x + 14.0,
        lounge.y + 28.0,
        19.0,
        TEXT,
    );
    let status = if game.special_table_busy {
        format!(
            "{:.0}s processing",
            (game.special_table_timer / 1000.0).max(0.0)
        )
    } else if let Some(customer) = ready_guest {
        format!("{} ready", ellipsize(&customer.display_name, 12))
    } else {
        "locked".to_string()
    };
    draw_text(
        &status,
        lounge.x + 14.0,
        lounge.y + 53.0,
        15.0,
        if is_active { LIGHTGRAY } else { MUTED },
    );
    if game.special_table_busy {
        draw_bar(
            lounge.x + 14.0,
            lounge.y + 70.0,
            lounge.w - 28.0,
            7.0,
            data.balance.special_table_process_time - game.special_table_timer,
            data.balance.special_table_process_time.max(1.0),
            SKYBLUE,
        );
    } else {
        draw_text(
            "VIP threshold service",
            lounge.x + 14.0,
            lounge.y + 76.0,
            12.0,
            MUTED,
        );
    }
}

pub(super) fn draw_dining_room(
    floor: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    now_ms: f64,
    selected_station: &Option<String>,
    textures: &HashMap<String, Texture2D>,
    ui: &mut UiActions,
) {
    draw_floor_pattern(floor);
    draw_rectangle_lines(floor.x, floor.y, floor.w, floor.h, 1.5, LINE);

    draw_text("Dining room", floor.x + 18.0, floor.y + 30.0, 24.0, TEXT);
    let selected_text = selected_station.as_deref().map(|color| {
        let ready = game
            .cooking_stations
            .get(color)
            .map(|station| station.dishes.len())
            .unwrap_or_default();
        if ready > 0 {
            format!("{} ready - click Serve", dish_label(data, color))
        } else {
            dish_label(data, color)
        }
    });
    draw_text(
        &format!(
            "Serving: {}",
            selected_text
                .unwrap_or_else(|| "No dish - cook, then click a ready station".to_string())
        ),
        floor.x + 18.0,
        floor.y + 54.0,
        16.0,
        MUTED,
    );

    let entrance_world = restaurant_entrance_position();
    let entrance = floor_to_screen(floor, entrance_world.0, entrance_world.1);
    draw_rectangle(
        entrance.x - 44.0,
        entrance.y - 28.0,
        88.0,
        56.0,
        Color::new(0.07, 0.10, 0.12, 1.0),
    );
    draw_rectangle_lines(
        entrance.x - 44.0,
        entrance.y - 28.0,
        88.0,
        56.0,
        1.5,
        SKYBLUE,
    );
    draw_text("Door", entrance.x - 18.0, entrance.y + 6.0, 16.0, SKYBLUE);

    draw_last_meal_lounge(floor, game, data);

    let max_tables = max_customer_count(data, progression);
    for table_index in 0..max_tables {
        let (x, y) = restaurant_table_position(table_index, max_tables);
        let table_customer = game
            .customers
            .iter()
            .find(|customer| customer.table_index == table_index);
        draw_table(
            floor_to_screen(floor, x, y),
            table_index,
            table_customer,
            data,
            progression,
            now_ms,
            selected_station,
            game,
            ui,
        );
    }

    if game.player.x >= 0.0 {
        draw_player_actor(
            floor_to_screen(floor, game.player.x, game.player.y),
            &game.player,
        );
    }

    let mut customers: Vec<_> = game.customers.iter().collect();
    customers.sort_by(|left, right| {
        left.floor_y
            .partial_cmp(&right.floor_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for customer in customers {
        draw_customer_sprite(floor, customer, data, selected_station, textures, game, ui);
    }

    if game.customers.is_empty() {
        draw_text(
            "Waiting for guests.",
            floor.x + 18.0,
            floor.y + floor.h - 28.0,
            18.0,
            MUTED,
        );
    }
}
