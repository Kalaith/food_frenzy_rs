use super::common::{draw_stat, BACKGROUND, MUTED, TEXT};
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
    let y = 22.0;
    draw_text("Feast Frenzy", 24.0, y + 31.0, 34.0, TEXT);
    draw_text("small cafe", 250.0, y + 31.0, 18.0, MUTED);

    let vip = if game.special_table_busy {
        format!("{:.0}s", (game.special_table_timer / 1000.0).max(0.0))
    } else {
        "ready".to_string()
    };
    let start_x = screen_width() - 890.0;
    draw_stat("Score", &game.score.to_string(), start_x, y, 150.0);
    draw_stat(
        "Cash",
        &progression.currency.to_string(),
        start_x + 162.0,
        y,
        130.0,
    );
    draw_stat(
        "Guests",
        &format!(
            "{}/{}",
            game.customers.len(),
            max_customer_count(data, progression)
        ),
        start_x + 304.0,
        y,
        130.0,
    );
    draw_stat(
        "Clientele",
        &format!(
            "{}/{}",
            progression.unlocked_customer_count(),
            data.customer_types.len()
        ),
        start_x + 446.0,
        y,
        150.0,
    );
    draw_stat("Lounge", &vip, start_x + 608.0, y, 110.0);
    draw_stat(
        "Prestige",
        &progression.prestige_level.to_string(),
        start_x + 730.0,
        y,
        130.0,
    );
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
    let width = screen_width();
    let height = screen_height();
    let margin = 24.0;
    let header_h = 76.0;
    let footer_h = 72.0;
    let gap = 16.0;
    let left_w = 340.0;
    let right_w = 360.0;
    let main_y = margin + header_h;
    let main_h = (height - main_y - footer_h - margin).max(520.0);
    let left = Rect::new(margin, main_y, left_w, main_h);
    let right = Rect::new(width - margin - right_w, main_y, right_w, main_h);
    let floor = Rect::new(
        left.x + left.w + gap,
        main_y,
        (right.x - left.x - left.w - gap * 2.0).max(640.0),
        main_h,
    );
    let feed = Rect::new(
        margin,
        height - footer_h + 10.0,
        width - margin * 2.0,
        footer_h - 22.0,
    );

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
