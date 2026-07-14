//! Draws the transient floating gain numbers spawned by gameplay: they rise
//! and fade above the spot where the value was earned, so every transaction
//! has visible cause and effect.

use super::common::{floor_to_screen, ACCENT, SUCCESS};
use crate::state::{FloaterAnchor, FloaterKind, GameState};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::{clamp01, ease_in_quad};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const MEAT_PINK: Color = Color::new(0.93, 0.52, 0.60, 1.0);
const BACKING: Color = Color::new(0.02, 0.02, 0.025, 1.0);
const RISE_PX: f32 = 46.0;

fn kind_color(kind: FloaterKind) -> Color {
    match kind {
        FloaterKind::Cash => SUCCESS,
        FloaterKind::Renown => ACCENT,
        FloaterKind::Meat => MEAT_PINK,
        FloaterKind::Alert => Color::new(0.94, 0.42, 0.36, 1.0),
    }
}

pub(super) fn draw_floaters(floor: Rect, game: &GameState) {
    let mut header_row = 0usize;
    for floater in &game.floaters.active {
        let progress = floater.progress();
        let alpha = clamp01(1.0 - ease_in_quad(progress));
        let font = 16.0;
        let dim = measure_ui_text(&floater.text, None, font as u16, 1.0);

        let (x, y) = match floater.anchor {
            FloaterAnchor::Floor { x, y } => {
                let screen = floor_to_screen(floor, x, y);
                (
                    screen.x - dim.width * 0.5,
                    screen.y - 96.0 - progress * RISE_PX,
                )
            }
            FloaterAnchor::Header => {
                let row = header_row;
                header_row += 1;
                (
                    screen_width() * 0.5 - dim.width * 0.5,
                    96.0 + row as f32 * 22.0 - progress * 12.0,
                )
            }
        };

        let color = with_alpha(kind_color(floater.kind), alpha);
        // Soft backing so the number reads over any floor art.
        draw_rectangle(
            x - 6.0,
            y - font + 2.0,
            dim.width + 12.0,
            font + 6.0,
            with_alpha(BACKING, 0.55 * alpha),
        );
        draw_ui_text(&floater.text, x, y, font, color);
    }
}
