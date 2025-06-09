mod constants;
mod game;
mod map_gen;
mod menu;
mod shader_types;
mod terrain;
mod display;
mod board_buffers;

use bevy::{
  dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
  log::{self, LogPlugin},
  prelude::*,
};

use constants::*;
use game::*;
use menu::menu::*;
use menu::*;
use std::sync::{Arc, Mutex};

#[derive(Resource, Clone, Debug)]
struct SharedGameState(Arc<Mutex<GameState>>);

use display::DisplayBoard;
use board_buffers::BoardBuffers;

use crate::display::{do_zoom, setup_display_board, zoom_out, DisplayMaterial};

fn main() {
  log::debug!("START");
  let mut app = App::new();
  let shared = Arc::new(Mutex::new(GameState::Splash));

  app
    .add_plugins((
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            resolution: (
              SIZE.0 as f32 * DISPLAY_FACTOR,
              SIZE.1 as f32 * DISPLAY_FACTOR,
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
        .set(ImagePlugin::default_nearest()),
      FpsOverlayPlugin {
        config: FpsOverlayConfig {
          text_config: TextFont {
            // Here we define size of our overlay
            font_size: 42.0,
            ..default()
          },
          // We can also change color of the overlay
          //text_color: OverlayColor::GREEN,
          // We can also set the refresh interval for the FPS counter
          refresh_interval: core::time::Duration::from_millis(100),
          enabled: true,
          ..default()
        },
      },
      // ZombieComputePlugin,
    ))
    .init_state::<GameState>()
    .insert_resource(SharedGameState(shared.clone()))
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(DisplayQuality::Medium)
    .insert_resource(Volume(7))
    // Declare the game state, whose starting value is determined by the `Default` trait
    .add_systems(Startup, setup)
    .add_plugins((
      splash::splash_plugin,
      menu_plugin,
      crate::game::ZombieComputePlugin,
    ));

  app.run();
}

fn setup(mut commands: Commands) {
  log::debug!("SETUP");

  commands.spawn(Camera2d);
}

