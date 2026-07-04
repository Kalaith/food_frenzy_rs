//! Feast Frenzy migration to Macroquad.

#![allow(clippy::too_many_arguments)]

mod app;
mod assets;
mod commands;
mod data;
mod engine;
mod gameplay;
mod lifecycle;
mod persistence;
mod player;
mod simulation;
mod state;
mod ui;

use macroquad::prelude::*;
use macroquad_toolkit::capture;

fn window_conf() -> Conf {
    capture::capture_window_conf("FOOD_FRENZY", "Feast Frenzy", 1920, 1080)
}

#[macroquad::main(window_conf)]
async fn main() {
    app::run().await;
}
