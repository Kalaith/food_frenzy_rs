//! Macroquad UI drawing and hitbox collection for Feast Frenzy.

use crate::data::{GameData, STATION_COLORS};
use crate::engine::{
    kitchen_pass_position, kitchen_station_position, max_customer_count, patience_multiplier,
    restaurant_entrance_position, restaurant_table_position, KITCHEN_SERVICE_LEFT,
    RESTAURANT_FLOOR_HEIGHT, RESTAURANT_FLOOR_WIDTH,
};
use crate::state::{Customer, GameState, ProgressionState};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{
    draw_badge as toolkit_draw_badge, draw_text_centered_in_box, progress_bar,
};
use std::collections::HashMap;

const BACKGROUND: Color = Color::new(0.055, 0.055, 0.065, 1.0);
const PANEL: Color = Color::new(0.085, 0.085, 0.105, 0.94);
const PANEL_SOFT: Color = Color::new(0.13, 0.13, 0.155, 1.0);
const TEXT: Color = Color::new(0.92, 0.91, 0.86, 1.0);
const MUTED: Color = Color::new(0.55, 0.56, 0.58, 1.0);
const LINE: Color = Color::new(0.28, 0.27, 0.26, 1.0);
const ACCENT: Color = Color::new(0.36, 0.56, 0.86, 1.0);

#[derive(Clone, Debug)]
pub struct UiActions {
    pub station_cook: Vec<(String, Rect)>,
    pub station_select: Vec<(String, Rect)>,
    pub serve_customer: HashMap<u32, Rect>,
    pub invite_customer: HashMap<u32, Rect>,
    pub upgrade_buttons: HashMap<String, Rect>,
    pub recipe_buttons: HashMap<String, Rect>,
    pub attract_buttons: HashMap<String, Rect>,
    pub prestige_button: Option<Rect>,
    pub clear_selection: Option<Rect>,
}

impl Default for UiActions {
    fn default() -> Self {
        Self {
            station_cook: Vec::new(),
            station_select: Vec::new(),
            serve_customer: HashMap::new(),
            invite_customer: HashMap::new(),
            upgrade_buttons: HashMap::new(),
            recipe_buttons: HashMap::new(),
            attract_buttons: HashMap::new(),
            prestige_button: None,
            clear_selection: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TitleAction {
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

#[derive(Clone, Debug)]
pub struct TitleActions {
    pub new_game: Rect,
    pub load_game: Rect,
    pub settings: Rect,
    pub exit: Rect,
}

impl TitleActions {
    pub fn action_at(&self, point: Vec2) -> Option<TitleAction> {
        if self.new_game.contains(point) {
            Some(TitleAction::NewGame)
        } else if self.load_game.contains(point) {
            Some(TitleAction::LoadGame)
        } else if self.settings.contains(point) {
            Some(TitleAction::Settings)
        } else if self.exit.contains(point) {
            Some(TitleAction::Exit)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    ToggleFullscreen,
    Back,
}

#[derive(Clone, Debug)]
pub struct SettingsActions {
    pub fullscreen_toggle: Rect,
    pub back: Rect,
}

impl SettingsActions {
    pub fn action_at(&self, point: Vec2) -> Option<SettingsAction> {
        if self.fullscreen_toggle.contains(point) {
            Some(SettingsAction::ToggleFullscreen)
        } else if self.back.contains(point) {
            Some(SettingsAction::Back)
        } else {
            None
        }
    }
}

fn station_label(color: &str) -> &'static str {
    match color {
        "blue" => "Blue",
        "green" => "Green",
        "yellow" => "Yellow",
        "red" => "Red",
        _ => "Dish",
    }
}

fn station_draw_color(color: &str) -> Color {
    match color {
        "blue" => SKYBLUE,
        "green" => LIME,
        "yellow" => YELLOW,
        "red" => ORANGE,
        _ => LIGHTGRAY,
    }
}

fn dish_label(data: &GameData, color: &str) -> String {
    data.dish_type_by_color(color)
        .map(|dish| dish.name.clone())
        .unwrap_or_else(|| station_label(color).to_string())
}

fn ellipsize(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut trimmed: String = text.chars().take(max_chars.saturating_sub(3)).collect();
    trimmed.push_str("...");
    trimmed
}

fn customer_fallback_color(customer_type: &str) -> Color {
    match customer_type {
        "pig" => PINK,
        "cow" => BEIGE,
        "sheep" => LIGHTGRAY,
        "rabbit" => WHITE,
        "cat" => GOLD,
        "deer" => BROWN,
        "duck" => YELLOW,
        "chicken" => ORANGE,
        "fish" => SKYBLUE,
        "fox" => RED,
        "goat" => LIME,
        "bear" => DARKBROWN,
        "monkey" => PURPLE,
        _ => MAGENTA,
    }
}

fn draw_panel(rect: Rect) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(PANEL)
        .with_border(1.0, Color::new(0.20, 0.20, 0.22, 1.0));
    macroquad_toolkit::ui::draw_surface(rect, &surface);
}

fn draw_section_title(text: &str, x: f32, y: f32) {
    draw_text(text, x, y, 22.0, TEXT);
}

fn draw_button(rect: Rect, text: &str, active: bool, disabled: bool) {
    let color = if disabled {
        Color::new(0.18, 0.18, 0.19, 1.0)
    } else if active {
        ACCENT
    } else {
        Color::new(0.25, 0.25, 0.27, 1.0)
    };
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(color)
        .with_border(1.0, if disabled { MUTED } else { TEXT });
    macroquad_toolkit::ui::draw_surface(rect, &surface);

    let font_size = if text.len() > 13 { 14.0 } else { 16.0 };
    draw_text_centered_in_box(
        text,
        rect.x + 6.0,
        rect.y,
        rect.w - 12.0,
        rect.h,
        font_size,
        if disabled { MUTED } else { WHITE },
    );
}

fn draw_menu_button(rect: Rect, text: &str) {
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let hovered = rect.contains(mouse);
    let color = if hovered {
        ACCENT
    } else {
        Color::new(0.16, 0.14, 0.13, 0.96)
    };
    let border = if hovered {
        WHITE
    } else {
        Color::new(0.78, 0.52, 0.30, 1.0)
    };
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(color).with_border(1.5, border);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_text_centered_in_box(
        text,
        rect.x + 8.0,
        rect.y,
        rect.w - 16.0,
        rect.h,
        (rect.h * 0.42).clamp(18.0, 24.0),
        WHITE,
    );
}

fn draw_title_background(title_texture: Option<&Texture2D>) {
    let width = screen_width();
    let height = screen_height();
    clear_background(Color::new(0.02, 0.018, 0.016, 1.0));

    if let Some(texture) = title_texture {
        let scale = (width / texture.width()).max(height / texture.height());
        let dest_size = vec2(texture.width() * scale, texture.height() * scale);
        let dest_x = (width - dest_size.x) * 0.5;
        let dest_y = (height - dest_size.y) * 0.5;
        draw_texture_ex(
            texture,
            dest_x,
            dest_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(dest_size),
                ..Default::default()
            },
        );
        draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.10));
    } else {
        draw_text_centered_in_box("Food Frenzy", 0.0, height * 0.22, width, 120.0, 72.0, TEXT);
    }

    let band_h = 170.0_f32.min(height * 0.28);
    draw_rectangle(
        0.0,
        height - band_h,
        width,
        band_h,
        Color::new(0.02, 0.018, 0.016, 0.78),
    );
    draw_rectangle(
        0.0,
        height - band_h,
        width,
        1.0,
        Color::new(0.80, 0.50, 0.25, 0.55),
    );
}

fn title_button_layout(width: f32, height: f32) -> TitleActions {
    let gap = 16.0;
    if width >= 860.0 {
        let button_w = ((width - 96.0 - gap * 3.0) / 4.0).clamp(168.0, 250.0);
        let total_w = button_w * 4.0 + gap * 3.0;
        let start_x = (width - total_w) * 0.5;
        let y = height - 108.0;
        return TitleActions {
            new_game: Rect::new(start_x, y, button_w, 54.0),
            load_game: Rect::new(start_x + (button_w + gap), y, button_w, 54.0),
            settings: Rect::new(start_x + (button_w + gap) * 2.0, y, button_w, 54.0),
            exit: Rect::new(start_x + (button_w + gap) * 3.0, y, button_w, 54.0),
        };
    }

    let button_w = (width - 64.0).clamp(220.0, 320.0);
    let button_h = 48.0;
    let start_x = (width - button_w) * 0.5;
    let start_y = (height - (button_h * 4.0 + gap * 3.0) - 24.0).max(120.0);
    TitleActions {
        new_game: Rect::new(start_x, start_y, button_w, button_h),
        load_game: Rect::new(start_x, start_y + (button_h + gap), button_w, button_h),
        settings: Rect::new(
            start_x,
            start_y + (button_h + gap) * 2.0,
            button_w,
            button_h,
        ),
        exit: Rect::new(
            start_x,
            start_y + (button_h + gap) * 3.0,
            button_w,
            button_h,
        ),
    }
}

pub fn draw_title_screen(title_texture: Option<&Texture2D>, status_message: &str) -> TitleActions {
    let width = screen_width();
    let height = screen_height();
    draw_title_background(title_texture);

    let actions = title_button_layout(width, height);
    if !status_message.is_empty() {
        draw_text_centered_in_box(
            status_message,
            24.0,
            actions.new_game.y - 42.0,
            width - 48.0,
            28.0,
            19.0,
            LIGHTGRAY,
        );
    }

    draw_menu_button(actions.new_game, "New Game");
    draw_menu_button(actions.load_game, "Load Game");
    draw_menu_button(actions.settings, "Settings");
    draw_menu_button(actions.exit, "Exit");

    actions
}

fn draw_toggle(rect: Rect, enabled: bool) {
    let bg = if enabled {
        Color::new(0.25, 0.58, 0.39, 1.0)
    } else {
        Color::new(0.25, 0.25, 0.27, 1.0)
    };
    let border = if enabled { LIME } else { MUTED };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);

    let knob_size = rect.h - 10.0;
    let knob_x = if enabled {
        rect.x + rect.w - knob_size - 5.0
    } else {
        rect.x + 5.0
    };
    draw_rectangle(
        knob_x,
        rect.y + 5.0,
        knob_size,
        knob_size,
        Color::new(0.94, 0.93, 0.86, 1.0),
    );

    draw_text_centered_in_box(
        if enabled { "On" } else { "Off" },
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        16.0,
        WHITE,
    );
}

pub fn draw_settings_screen(fullscreen_enabled: bool) -> SettingsActions {
    let width = screen_width();
    let height = screen_height();
    clear_background(BACKGROUND);

    draw_rectangle(0.0, 0.0, width, 96.0, Color::new(0.075, 0.065, 0.06, 1.0));
    draw_rectangle(0.0, 95.0, width, 1.0, Color::new(0.80, 0.50, 0.25, 0.55));
    draw_text("Settings", 32.0, 60.0, 36.0, TEXT);

    let content_w = width.min(720.0);
    let content_x = (width - content_w) * 0.5;
    let row = Rect::new(content_x + 24.0, height * 0.32, content_w - 48.0, 78.0);
    let row_surface = macroquad_toolkit::ui::SurfaceStyle::new(PANEL)
        .with_border(1.0, Color::new(0.20, 0.20, 0.22, 1.0));
    macroquad_toolkit::ui::draw_surface(row, &row_surface);
    draw_text("Fullscreen", row.x + 24.0, row.y + 48.0, 24.0, TEXT);

    let toggle = Rect::new(row.x + row.w - 142.0, row.y + 16.0, 112.0, 46.0);
    draw_toggle(toggle, fullscreen_enabled);

    let back = Rect::new(content_x + 24.0, height - 112.0, 180.0, 54.0);
    draw_menu_button(back, "Back");

    SettingsActions {
        fullscreen_toggle: row,
        back,
    }
}

fn draw_badge(rect: Rect, text: &str, color: Color) {
    toolkit_draw_badge(rect, text, color, WHITE);
}

fn draw_card(rect: Rect, title: &str) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.105, 0.105, 0.125, 1.0))
        .with_border(1.0, LINE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_text(title, rect.x + 12.0, rect.y + 25.0, 19.0, TEXT);
}

fn draw_tooltip(text: &str, center_x: f32, y: f32) {
    let text_dim = measure_text(text, None, 14, 1.0);
    let rect = Rect::new(
        center_x - text_dim.width * 0.5 - 10.0,
        y,
        text_dim.width + 20.0,
        24.0,
    );
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.02, 0.02, 0.025, 0.86))
        .with_border(1.0, SKYBLUE);
    macroquad_toolkit::ui::draw_surface(rect, &surface);
    draw_text(text, rect.x + 10.0, rect.y + 17.0, 14.0, TEXT);
}

fn draw_bar(x: f32, y: f32, width: f32, height: f32, value: f32, max_value: f32, color: Color) {
    progress_bar(x, y, width, height, value, max_value, color);
}

fn draw_stat(label: &str, value: &str, x: f32, y: f32, w: f32) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Color::new(0.10, 0.10, 0.12, 1.0));
    macroquad_toolkit::ui::draw_surface(Rect::new(x, y, w, 44.0), &surface);
    draw_text(label, x + 12.0, y + 17.0, 13.0, MUTED);
    draw_text(value, x + 12.0, y + 36.0, 18.0, TEXT);
}

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

fn sorted_ingredient_lines(game: &GameState) -> Vec<String> {
    let mut ingredients: Vec<_> = game
        .ingredients
        .iter()
        .filter(|(name, amount)| name.as_str() != "regular" && **amount > 0)
        .map(|(name, amount)| format!("{name}: {amount}"))
        .collect();
    ingredients.sort();
    ingredients
}

fn format_unlock_cost(cost: &HashMap<String, i64>) -> String {
    if cost.is_empty() {
        return "Open".to_string();
    }

    let mut parts: Vec<_> = cost
        .iter()
        .map(|(ingredient, amount)| format!("{amount} {ingredient}"))
        .collect();
    parts.sort();
    parts.join(", ")
}

fn can_afford_cost(game: &GameState, cost: &HashMap<String, i64>) -> bool {
    cost.iter()
        .all(|(ingredient, amount)| game.has_ing(ingredient, *amount))
}

fn player_near_customer(game: &GameState, customer: &Customer, range: f32) -> bool {
    if game.player.x < 0.0 || !customer.is_seated {
        return false;
    }
    let dx = game.player.x - customer.floor_x;
    let dy = game.player.y - customer.floor_y;
    (dx * dx + dy * dy).sqrt() <= range
}

fn patience_limit_ms(customer: &Customer, data: &GameData, progression: &ProgressionState) -> f32 {
    let traits = customer.traits(data);
    let mut patience = data.balance.customer_patience_time * patience_multiplier(progression);
    if traits.fast_spoilage {
        patience *= 0.55;
    }
    patience.max(1.0)
}

fn patience_remaining_ratio(
    customer: &Customer,
    data: &GameData,
    progression: &ProgressionState,
    now_ms: f64,
) -> f32 {
    let elapsed = (now_ms - customer.arrived_at_ms).max(0.0) as f32;
    (1.0 - elapsed / patience_limit_ms(customer, data, progression)).clamp(0.0, 1.0)
}

fn patience_color(ratio: f32) -> Color {
    if ratio < 0.24 {
        RED
    } else if ratio < 0.48 {
        ORANGE
    } else {
        LIME
    }
}

fn floor_to_screen(floor: Rect, world_x: f32, world_y: f32) -> Vec2 {
    vec2(
        floor.x + (world_x / RESTAURANT_FLOOR_WIDTH) * floor.w,
        floor.y + (world_y / RESTAURANT_FLOOR_HEIGHT) * floor.h,
    )
}

fn kitchen_to_screen(panel: Rect, world_x: f32, world_y: f32) -> Vec2 {
    let x_t = ((world_x - KITCHEN_SERVICE_LEFT) / -KITCHEN_SERVICE_LEFT).clamp(0.0, 1.0);
    let y_t = (world_y / RESTAURANT_FLOOR_HEIGHT).clamp(0.0, 1.0);
    vec2(
        panel.x + 22.0 + x_t * (panel.w - 44.0),
        panel.y + y_t * panel.h,
    )
}

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

fn customer_label(customer: &crate::state::Customer, data: &GameData) -> String {
    let customer_type = data
        .customer_type_by_id(&customer.customer_type)
        .map(|item| item.name.replace(" Girl", ""))
        .unwrap_or_else(|| "Guest".to_string());
    format!("{} / {}", customer.display_name, customer_type)
}

fn customer_order_text(customer: &Customer, data: &GameData) -> String {
    data.customer_type_by_id(&customer.customer_type)
        .map(|customer_type| {
            let labels: Vec<String> = customer_type
                .preferred_dishes
                .iter()
                .take(2)
                .map(|color| dish_label(data, color))
                .collect();
            if labels.is_empty() {
                "Any house plate".to_string()
            } else {
                labels.join(" + ")
            }
        })
        .unwrap_or_else(|| "Any house plate".to_string())
}

fn draw_player_actor(pos: Vec2, player: &crate::state::PlayerActor) {
    draw_player_actor_scaled(pos, player, 1.0);
}

fn draw_player_actor_scaled(pos: Vec2, player: &crate::state::PlayerActor, scale: f32) {
    let shadow = 24.0 * scale;
    let body_w = 28.0 * scale;
    let body_h = 36.0 * scale;
    let apron_w = 20.0 * scale;
    let apron_h = 28.0 * scale;
    let head = 14.0 * scale;
    let hat_w = 36.0 * scale;
    let hat_h = 10.0 * scale;
    draw_circle(
        pos.x,
        pos.y - 3.0 * scale,
        shadow,
        Color::new(0.02, 0.02, 0.025, 0.42),
    );
    draw_rectangle(
        pos.x - body_w * 0.5,
        pos.y - 42.0 * scale,
        body_w,
        body_h,
        Color::new(0.78, 0.78, 0.72, 1.0),
    );
    draw_rectangle(
        pos.x - apron_w * 0.5,
        pos.y - 34.0 * scale,
        apron_w,
        apron_h,
        Color::new(0.18, 0.20, 0.23, 1.0),
    );
    draw_circle(
        pos.x,
        pos.y - 51.0 * scale,
        head,
        Color::new(0.79, 0.62, 0.48, 1.0),
    );
    draw_rectangle(
        pos.x - hat_w * 0.5,
        pos.y - 68.0 * scale,
        hat_w,
        hat_h,
        Color::new(0.94, 0.92, 0.86, 1.0),
    );
    draw_circle(
        pos.x - 8.0 * scale,
        pos.y - 67.0 * scale,
        8.0 * scale,
        Color::new(0.94, 0.92, 0.86, 1.0),
    );
    draw_circle(
        pos.x + 4.0 * scale,
        pos.y - 71.0 * scale,
        9.0 * scale,
        Color::new(0.94, 0.92, 0.86, 1.0),
    );
    draw_rectangle_lines(
        pos.x - body_w * 0.5,
        pos.y - 42.0 * scale,
        body_w,
        body_h,
        1.0,
        LINE,
    );

    if let Some(station) = &player.carried_station {
        draw_circle(
            pos.x + 19.0 * scale,
            pos.y - 31.0 * scale,
            9.0 * scale,
            station_draw_color(station),
        );
        draw_circle_lines(
            pos.x + 19.0 * scale,
            pos.y - 31.0 * scale,
            9.0 * scale,
            1.5,
            WHITE,
        );
    }

    if scale >= 0.85 {
        let label = "You";
        let font_size = (13.0 * scale).round() as u16;
        let text_dim = measure_text(label, None, font_size, 1.0);
        draw_rectangle(
            pos.x - text_dim.width * 0.5 - 7.0,
            pos.y - 91.0 * scale,
            text_dim.width + 14.0,
            20.0 * scale,
            Color::new(0.02, 0.02, 0.025, 0.72),
        );
        draw_text(
            label,
            pos.x - text_dim.width * 0.5,
            pos.y - 77.0 * scale,
            font_size as f32,
            TEXT,
        );
    }

    if player.action_lock_ms > 0.0 {
        draw_tooltip("Cooking...", pos.x, pos.y - 106.0 * scale);
        draw_bar(
            pos.x - 26.0 * scale,
            pos.y - 13.0 * scale,
            52.0 * scale,
            5.0 * scale,
            player.action_lock_ms,
            900.0,
            ORANGE,
        );
    }
}

fn draw_customer_sprite(
    floor: Rect,
    customer: &crate::state::Customer,
    data: &GameData,
    selected_station: &Option<String>,
    textures: &HashMap<String, Texture2D>,
    game: &GameState,
    ui: &mut UiActions,
) {
    let pos = floor_to_screen(floor, customer.floor_x, customer.floor_y);
    let sprite_rect = Rect::new(pos.x - 34.0, pos.y - 82.0, 68.0, 80.0);
    let can_serve = selected_station
        .as_ref()
        .and_then(|station| game.cooking_stations.get(station))
        .is_some_and(|station| !station.dishes.is_empty())
        && customer.is_seated;

    draw_circle(
        pos.x,
        pos.y - 2.0,
        28.0,
        Color::new(0.02, 0.02, 0.025, 0.38),
    );
    if let Some(texture) = textures.get(&customer.customer_type) {
        draw_texture_ex(
            texture,
            sprite_rect.x,
            sprite_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(sprite_rect.w, sprite_rect.h)),
                ..Default::default()
            },
        );
    } else {
        draw_circle(
            pos.x,
            pos.y - 43.0,
            26.0,
            customer_fallback_color(&customer.customer_type),
        );
    }

    let label = customer_label(customer, data);
    let text_dim = measure_text(&label, None, 16, 1.0);
    let label_w = text_dim.width + 22.0;
    draw_rectangle(
        pos.x - label_w * 0.5,
        pos.y - 116.0,
        label_w,
        38.0,
        Color::new(0.02, 0.02, 0.025, 0.78),
    );
    draw_text(
        &label,
        pos.x - label_w * 0.5 + 11.0,
        pos.y - 95.0,
        16.0,
        TEXT,
    );
    draw_bar(
        pos.x - 52.0,
        pos.y - 72.0,
        104.0,
        6.0,
        customer.total_satisfaction,
        customer.max_satisfaction.total(),
        LIME,
    );
    if customer.is_seated {
        let order = ellipsize(&customer_order_text(customer, data), 27);
        let order_dim = measure_text(&order, None, 13, 1.0);
        draw_rectangle(
            pos.x - order_dim.width * 0.5 - 8.0,
            pos.y - 64.0,
            order_dim.width + 16.0,
            18.0,
            Color::new(0.02, 0.02, 0.025, 0.70),
        );
        draw_text(
            &order,
            pos.x - order_dim.width * 0.5,
            pos.y - 51.0,
            13.0,
            LIGHTGRAY,
        );
    }

    if can_serve {
        draw_rectangle_lines(
            sprite_rect.x,
            sprite_rect.y,
            sprite_rect.w,
            sprite_rect.h,
            2.0,
            SKYBLUE,
        );
        if player_near_customer(game, customer, 118.0) {
            draw_tooltip("Space / E: Serve", pos.x, pos.y - 145.0);
        }
        ui.serve_customer.entry(customer.id).or_insert(sprite_rect);
    }

    if customer.is_seated && crate::engine::can_process_customer(customer, data) {
        let invite_rect = Rect::new(pos.x - 36.0, pos.y + 22.0, 72.0, 28.0);
        draw_button(invite_rect, "VIP", true, false);
        ui.invite_customer.insert(customer.id, invite_rect);
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

fn draw_dining_room(
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

    let entrance = floor_to_screen(
        floor,
        restaurant_entrance_position().0,
        restaurant_entrance_position().1,
    );
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

fn draw_kitchen(
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
        let Some(station) = game.cooking_stations.get(color) else {
            continue;
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
            let cook_time = data
                .dish_type_by_color(color)
                .map(|dish| dish.cook_time_ms)
                .unwrap_or(1.0);
            let progress = 1.0 - (station.remaining_ms / cook_time).clamp(0.0, 1.0);
            draw_bar(
                pad_rect.x + 4.0,
                pad_rect.y + pad_rect.h - 6.0,
                pad_rect.w - 8.0,
                4.0,
                progress,
                1.0,
                station_draw_color(color),
            );
        }
        if ready > 0 {
            draw_badge(
                Rect::new(pad_rect.x + 24.0, pad_rect.y - 10.0, 26.0, 20.0),
                &ready.to_string(),
                station_draw_color(color),
            );
        }
        if ready > 0 {
            ui.station_select.push((color.to_string(), pad_rect));
        } else if can_cook {
            ui.station_cook.push((color.to_string(), pad_rect));
        }
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
        let Some(station) = game.cooking_stations.get(color) else {
            continue;
        };
        let is_selected = selected_station.as_deref() == Some(color);
        let ready = station.dishes.len();
        let row = Rect::new(panel.x + 12.0, y, panel.w - 24.0, row_h - 10.0);
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
        let label = dish_label(data, color);
        draw_text(&label, row.x + 34.0, row.y + 31.0, 18.0, TEXT);
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
            let cook_time = data
                .dish_type_by_color(color)
                .map(|dish| dish.cook_time_ms)
                .unwrap_or(1.0);
            let progress = 1.0 - (station.remaining_ms / cook_time).clamp(0.0, 1.0);
            draw_bar(
                row.x + 34.0,
                row.y + 65.0,
                row.w - 58.0,
                5.0,
                progress,
                1.0,
                station_draw_color(color),
            );
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

fn draw_growth_panel(
    panel: Rect,
    game: &GameState,
    progression: &ProgressionState,
    data: &GameData,
    ui: &mut UiActions,
) {
    draw_panel(panel);
    draw_section_title("Cafe plan", panel.x + 18.0, panel.y + 30.0);

    let card_x = panel.x + 14.0;
    let card_w = panel.w - 28.0;
    let guest_card = Rect::new(card_x, panel.y + 48.0, card_w, 286.0);
    let upgrade_card = Rect::new(card_x, guest_card.y + guest_card.h + 12.0, card_w, 178.0);
    let recipe_card = Rect::new(
        card_x,
        upgrade_card.y + upgrade_card.h + 12.0,
        card_w,
        panel.y + panel.h - (upgrade_card.y + upgrade_card.h + 26.0),
    );

    draw_card(guest_card, "Guests");
    let ingredients = sorted_ingredient_lines(game);
    draw_text(
        "Pantry",
        guest_card.x + 12.0,
        guest_card.y + 54.0,
        14.0,
        MUTED,
    );
    if ingredients.is_empty() {
        draw_text(
            "Empty",
            guest_card.x + 68.0,
            guest_card.y + 54.0,
            14.0,
            MUTED,
        );
    } else {
        let pantry = ingredients
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("  ");
        draw_text(
            &ellipsize(&pantry, 34),
            guest_card.x + 68.0,
            guest_card.y + 54.0,
            14.0,
            LIGHTGRAY,
        );
    }

    draw_text("Next", guest_card.x + 12.0, guest_card.y + 88.0, 15.0, TEXT);
    let mut locked_customer_types: Vec<_> = data
        .customer_types
        .iter()
        .filter(|customer_type| !progression.is_customer_unlocked(&customer_type.id))
        .collect();
    locked_customer_types.sort_by(|left, right| {
        left.profile_tier
            .cmp(&right.profile_tier)
            .then_with(|| left.name.cmp(&right.name))
    });
    if locked_customer_types.is_empty() {
        draw_text(
            "All known guests unlocked",
            guest_card.x + 12.0,
            guest_card.y + 116.0,
            15.0,
            MUTED,
        );
    } else {
        let mut y = guest_card.y + 116.0;
        for customer_type in locked_customer_types.iter().take(3) {
            let can_attract = can_afford_cost(game, &customer_type.unlock_cost);
            draw_text(
                &format!(
                    "T{} {}",
                    customer_type.profile_tier.max(1),
                    ellipsize(&customer_type.name, 18)
                ),
                guest_card.x + 12.0,
                y,
                16.0,
                if can_attract { TEXT } else { MUTED },
            );
            draw_text(
                &ellipsize(&format_unlock_cost(&customer_type.unlock_cost), 26),
                guest_card.x + 12.0,
                y + 19.0,
                12.0,
                if can_attract { LIGHTGRAY } else { MUTED },
            );
            let button_rect = Rect::new(guest_card.x + guest_card.w - 94.0, y - 18.0, 78.0, 28.0);
            draw_button(button_rect, "Attract", can_attract, !can_attract);
            ui.attract_buttons
                .insert(customer_type.id.clone(), button_rect);
            y += 58.0;
        }
    }

    draw_card(upgrade_card, "Upgrades");
    let mut y = upgrade_card.y + 58.0;
    for upgrade in progression.upgrades.iter().take(2) {
        let can_buy = progression.currency >= upgrade.cost && upgrade.level < upgrade.max_level;
        draw_text(
            &format!(
                "{}  L{}/{}",
                ellipsize(&upgrade.name, 20),
                upgrade.level,
                upgrade.max_level
            ),
            upgrade_card.x + 12.0,
            y,
            15.0,
            if can_buy { TEXT } else { LIGHTGRAY },
        );
        draw_text(
            &format!("${}", upgrade.cost),
            upgrade_card.x + 12.0,
            y + 18.0,
            12.0,
            MUTED,
        );
        let button_rect = Rect::new(upgrade_card.x + upgrade_card.w - 94.0, y - 18.0, 78.0, 28.0);
        draw_button(button_rect, "Buy", can_buy, !can_buy);
        ui.upgrade_buttons.insert(upgrade.id.clone(), button_rect);
        y += 50.0;
    }

    draw_card(recipe_card, "Recipes");
    let mut recipe_y = recipe_card.y + 58.0;
    for recipe in progression.recipes.iter().take(2) {
        let button_rect = Rect::new(
            recipe_card.x + recipe_card.w - 94.0,
            recipe_y - 18.0,
            78.0,
            28.0,
        );
        draw_text(
            &ellipsize(&recipe.name, 22),
            recipe_card.x + 12.0,
            recipe_y,
            15.0,
            if recipe.unlocked { TEXT } else { MUTED },
        );
        draw_button(
            button_rect,
            if recipe.unlocked { "Craft" } else { "Locked" },
            recipe.unlocked,
            !recipe.unlocked,
        );
        ui.recipe_buttons.insert(recipe.id.clone(), button_rect);
        recipe_y += 46.0;
    }

    let prestige_rect = Rect::new(
        recipe_card.x + 12.0,
        recipe_card.y + recipe_card.h - 42.0,
        recipe_card.w - 24.0,
        32.0,
    );
    let can_prestige = progression.total_score >= data.balance.prestige_score_requirement;
    draw_button(
        prestige_rect,
        &format!("Prestige +{}", progression.prestige_reward()),
        can_prestige,
        !can_prestige,
    );
    ui.prestige_button = Some(prestige_rect);
}

fn draw_event_feed(rect: Rect, game: &GameState) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.07, 0.07, 0.085, 0.95),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        Color::new(0.18, 0.18, 0.20, 1.0),
    );
    draw_text("Ticker", rect.x + 14.0, rect.y + 24.0, 18.0, TEXT);

    let mut x = rect.x + 92.0;
    for message in game.messages.iter().rev().take(5) {
        let text = ellipsize(message, 38);
        let text_dim = measure_text(&text, None, 15, 1.0);
        let pill_w = (text_dim.width + 28.0).min(330.0);
        if x + pill_w > rect.x + rect.w - 14.0 {
            break;
        }
        draw_rectangle(
            x,
            rect.y + 10.0,
            pill_w,
            30.0,
            Color::new(0.11, 0.11, 0.13, 1.0),
        );
        draw_text(&text, x + 12.0, rect.y + 30.0, 15.0, LIGHTGRAY);
        x += pill_w + 12.0;
    }
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
