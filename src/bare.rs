use bevy::{
  log::{self, LogPlugin},
  prelude::*,
};
fn main() {
  App::new()
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          primary_window: Some(Window {
            title: "Game Of Life".to_string(),
            resolution: [1200.0, 800.0].into(),
            ..default()
          }),
          ..default()
        })
        .set(LogPlugin {
          filter: "info,wgpu_core=warn,wgpu_hal=warn,bevy_jam6=debug".into(),
          level: bevy::log::Level::DEBUG,
          ..default()
        }),
    )
    .add_systems(Startup, (setup))
    .run();
}

fn setup(mut commands: Commands) {
  log::debug!("RUNNING");
}
