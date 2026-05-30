use super::common::{draw_resource_tile, BACKGROUND, GOLD, LINE};
use super::dining::draw_dining_room;
use super::growth::{draw_event_feed, draw_growth_panel};
use super::kitchen::draw_kitchen;
use super::types::UiActions;
use crate::data::GameData;
use crate::engine::max_customer_count;
use crate::state::{GameState, ProgressionState};
use macroquad::prelude::*;
use std::collections::HashMap;

fn draw_top_header(data: &GameData, game: &GameState, progression: &ProgressionState) {
    let width = screen_width();
    let bar = Rect::new(8.0, 8.0, width - 16.0, 70.0);
    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        Color::new(0.045, 0.038, 0.044, 1.0),
    );
    draw_rectangle_lines(bar.x, bar.y, bar.w, bar.h, 1.0, LINE);
    draw_rectangle_lines(
        bar.x + 3.0,
        bar.y + 3.0,
        bar.w - 6.0,
        bar.h - 6.0,
        1.0,
        GOLD,
    );

    let vip = if game.special_table_busy {
        format!("{:.0}s", (game.special_table_timer / 1000.0).max(0.0))
    } else {
        "ready".to_string()
    };
    let tiles = [
        (
            "Cash",
            progression.currency.to_string(),
            Color::new(0.52, 0.84, 0.46, 1.0),
        ),
        (
            "Score",
            game.score.to_string(),
            Color::new(0.44, 0.66, 0.96, 1.0),
        ),
        (
            "Guests",
            format!(
                "{}/{}",
                game.customers.len(),
                max_customer_count(data, progression)
            ),
            Color::new(0.90, 0.70, 0.40, 1.0),
        ),
        (
            "Clientele",
            format!(
                "{}/{}",
                progression.unlocked_customer_count(),
                data.customer_types.len()
            ),
            Color::new(0.58, 0.76, 0.54, 1.0),
        ),
        ("Lounge", vip, Color::new(0.74, 0.52, 0.92, 1.0)),
        (
            "Prestige",
            progression.prestige_level.to_string(),
            Color::new(0.82, 0.45, 0.88, 1.0),
        ),
    ];
    let gap = 8.0;
    let tile_w = (bar.w - 24.0 - gap * (tiles.len() as f32 - 1.0)) / tiles.len() as f32;
    let mut x = bar.x + 12.0;
    for (label, value, accent) in tiles {
        draw_resource_tile(
            Rect::new(x, bar.y + 8.0, tile_w, 54.0),
            label,
            &value,
            Some(accent),
        );
        x += tile_w + gap;
    }
}

fn layout_rects(width: f32, height: f32) -> (Rect, Rect, Rect, Rect) {
    let margin = 8.0;
    let header_h = 78.0;
    let footer_h = 56.0;
    let gap = 8.0;
    let min_floor_w = 420.0;
    let preferred_side_w = (width * 0.225).clamp(320.0, 420.0);
    let side_budget = (width - margin * 2.0 - gap * 2.0 - min_floor_w).max(0.0) * 0.5;
    let side_w = preferred_side_w.min(side_budget).clamp(260.0, 420.0);
    let left_w = side_w;
    let right_w = side_w;
    let main_y = margin + header_h;
    let main_h = (height - main_y - footer_h - margin).max(520.0);
    let left = Rect::new(margin, main_y, left_w, main_h);
    let right = Rect::new(width - margin - right_w, main_y, right_w, main_h);
    let floor = Rect::new(
        left.x + left.w + gap,
        main_y,
        (right.x - left.x - left.w - gap * 2.0).max(min_floor_w),
        main_h,
    );
    let feed = Rect::new(
        margin,
        height - footer_h + 8.0,
        width - margin * 2.0,
        footer_h - 14.0,
    );
    (left, floor, right, feed)
}

pub fn draw_and_collect_hitboxes(
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    now_ms: f64,
    selected_station: &Option<String>,
    character_textures: &HashMap<String, Texture2D>,
) -> UiActions {
    let mut ui = UiActions::default();
    let (left, floor, right, feed) = layout_rects(screen_width(), screen_height());

    clear_background(BACKGROUND);
    draw_top_header(data, game, progression);
    draw_kitchen(left, game, data, selected_station, &mut ui);
    draw_dining_room(
        floor,
        game,
        progression,
        data,
        now_ms,
        selected_station,
        character_textures,
        &mut ui,
    );
    draw_growth_panel(right, game, progression, data, &mut ui);
    draw_event_feed(feed, game);

    ui
}
