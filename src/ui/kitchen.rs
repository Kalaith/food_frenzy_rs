use super::actors::draw_player_actor_scaled;
use super::common::{
    dish_label, draw_badge, draw_bar, draw_button, draw_panel, draw_section_title,
    kitchen_to_screen, station_draw_color, LINE, MUTED, PANEL_SOFT, TEXT,
};
use super::types::UiActions;
use crate::data::{GameData, STATION_COLORS};
use crate::engine::{kitchen_pass_position, kitchen_station_position};
use crate::state::GameState;
use macroquad::prelude::*;

pub(super) fn draw_kitchen(
    panel: Rect,
    game: &GameState,
    data: &GameData,
    selected_station: &Option<String>,
    ui: &mut UiActions,
) {
    draw_panel(panel);
    draw_section_title("Kitchen", panel.x + 18.0, panel.y + 30.0);

    let counter = Rect::new(panel.x + 12.0, panel.y + 48.0, panel.w - 24.0, 172.0);
    draw_rectangle(
        counter.x,
        counter.y,
        counter.w,
        counter.h,
        Color::new(0.105, 0.10, 0.095, 1.0),
    );
    draw_rectangle_lines(counter.x, counter.y, counter.w, counter.h, 1.0, LINE);
    draw_line(
        counter.x + 24.0,
        counter.y + counter.h - 40.0,
        counter.x + counter.w - 24.0,
        counter.y + counter.h - 40.0,
        3.0,
        Color::new(0.24, 0.22, 0.20, 1.0),
    );

    let pass = kitchen_pass_position();
    let pass_pos = kitchen_to_screen(panel, pass.0, pass.1);
    draw_rectangle(
        pass_pos.x - 24.0,
        pass_pos.y - 13.0,
        48.0,
        26.0,
        Color::new(0.16, 0.17, 0.18, 1.0),
    );
    draw_rectangle_lines(pass_pos.x - 24.0, pass_pos.y - 13.0, 48.0, 26.0, 1.0, LINE);

    for color in STATION_COLORS {
        draw_station_pad(panel, color, game, data, selected_station, ui);
    }

    if game.player.x < 0.0 {
        draw_player_actor_scaled(
            kitchen_to_screen(panel, game.player.x, game.player.y),
            &game.player,
            0.72,
        );
    }

    let row_h = 82.0;
    let mut y = panel.y + 238.0;
    for color in STATION_COLORS {
        if let Some(station) = game.cooking_stations.get(color) {
            let row = Rect::new(panel.x + 12.0, y, panel.w - 24.0, row_h - 10.0);
            let is_selected = selected_station.as_deref() == Some(color);
            draw_station_row(row, color, station, is_selected, data);
        }
        y += row_h;
    }

    let clear_rect = Rect::new(
        panel.x + 18.0,
        panel.y + panel.h - 48.0,
        panel.w - 36.0,
        30.0,
    );
    draw_button(clear_rect, "Clear selection", false, false);
    ui.clear_selection = Some(clear_rect);
}

fn draw_station_pad(
    panel: Rect,
    color: &str,
    game: &GameState,
    data: &GameData,
    selected_station: &Option<String>,
    ui: &mut UiActions,
) {
    let Some(station) = game.cooking_stations.get(color) else {
        return;
    };
    let pad_world = kitchen_station_position(color);
    let pad_pos = kitchen_to_screen(panel, pad_world.0, pad_world.1);
    let ready = station.dishes.len();
    let is_selected = selected_station.as_deref() == Some(color);
    let can_cook = station.can_cook(data.balance.cooking_slots_limit);
    let pad_rect = Rect::new(pad_pos.x - 23.0, pad_pos.y - 20.0, 46.0, 40.0);
    draw_rectangle(
        pad_rect.x,
        pad_rect.y,
        pad_rect.w,
        pad_rect.h,
        if is_selected {
            Color::new(0.18, 0.23, 0.31, 1.0)
        } else {
            Color::new(0.15, 0.14, 0.13, 1.0)
        },
    );
    draw_circle(pad_pos.x, pad_pos.y - 2.0, 9.0, station_draw_color(color));
    draw_rectangle_lines(
        pad_rect.x,
        pad_rect.y,
        pad_rect.w,
        pad_rect.h,
        1.0,
        if ready > 0 || can_cook { TEXT } else { LINE },
    );
    if station.is_cooking {
        draw_cooking_bar(
            Rect::new(
                pad_rect.x + 4.0,
                pad_rect.y + pad_rect.h - 6.0,
                pad_rect.w - 8.0,
                4.0,
            ),
            color,
            station.remaining_ms,
            data,
        );
    }
    if ready > 0 {
        draw_badge(
            Rect::new(pad_rect.x + 24.0, pad_rect.y - 10.0, 26.0, 20.0),
            &ready.to_string(),
            station_draw_color(color),
        );
        ui.station_select.push((color.to_string(), pad_rect));
    } else if can_cook {
        ui.station_cook.push((color.to_string(), pad_rect));
    }
}

fn draw_station_row(
    row: Rect,
    color: &str,
    station: &crate::state::CookingStation,
    is_selected: bool,
    data: &GameData,
) {
    let ready = station.dishes.len();
    draw_rectangle(
        row.x,
        row.y,
        row.w,
        row.h,
        if is_selected {
            Color::new(0.16, 0.21, 0.29, 1.0)
        } else {
            PANEL_SOFT
        },
    );

    draw_circle(row.x + 18.0, row.y + 25.0, 7.0, station_draw_color(color));
    draw_text(
        &dish_label(data, color),
        row.x + 34.0,
        row.y + 31.0,
        18.0,
        TEXT,
    );
    let ready_label = if ready > 0 {
        format!("{ready} ready - click to carry")
    } else {
        "0 ready".to_string()
    };
    draw_text(
        &ready_label,
        row.x + 34.0,
        row.y + 52.0,
        16.0,
        if ready > 0 { WHITE } else { MUTED },
    );

    if station.is_cooking {
        draw_cooking_bar(
            Rect::new(row.x + 34.0, row.y + 65.0, row.w - 58.0, 5.0),
            color,
            station.remaining_ms,
            data,
        );
    }
}

fn draw_cooking_bar(rect: Rect, color: &str, remaining_ms: f32, data: &GameData) {
    let cook_time = data
        .dish_type_by_color(color)
        .map(|dish| dish.cook_time_ms)
        .unwrap_or(1.0);
    let progress = 1.0 - (remaining_ms / cook_time).clamp(0.0, 1.0);
    draw_bar(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        progress,
        1.0,
        station_draw_color(color),
    );
}
