mod game;
mod menu;

use bevy::{
    log::{self, LogPlugin},
    prelude::*
};

use game::*;
use menu::menu::*;
use menu::*;
use std::sync::{Arc, Mutex};

#[derive(Resource, Clone, Debug)]
struct SharedGameState(Arc<Mutex<GameState>>);

fn main() {
    log::debug!("START");
    let mut app = App::new();
    app.add_plugins((DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (
                    (SIZE.0 * DISPLAY_FACTOR) as f32,
                    (SIZE.1 * DISPLAY_FACTOR) as f32,
                )
                    .into(),
                // uncomment for unthrottled FPS
                // present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        })
        .set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,bevy_jam6=debug".into(),
            level: bevy::log::Level::DEBUG,
            ..default()
        })
        .set(ImagePlugin::default_nearest()),))
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        // Declare the game state, whose starting value is determined by the `Default` trait
        .add_systems(Startup, setup)
        .add_plugins((splash::splash_plugin, menu_plugin, GameOfLifeComputePlugin));

    app.run();
}

fn setup(mut commands: Commands) {
    log::debug!("SETUP");

    commands.spawn(Camera2d);
}
