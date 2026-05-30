//! Feast Frenzy migration to Macroquad.

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

fn window_conf() -> Conf {
    Conf {
        window_title: "Feast Frenzy".to_owned(),
        window_width: 1920,
        window_height: 1080,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    app::run().await;
}
