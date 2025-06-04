use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use bevy_life::{
  ConwayCellState, GameOfLife2dPlugin, MooreCell2d, SimulationBatch,
};
use rand::Rng;

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
    //.add_plugins(GameOfLife2dPlugin::default())
    //.insert_resource(SimulationBatch)
    .add_systems(Startup, (setup_camera, setup_test))
    .add_systems(Update, update)
    .run();
}

fn update(mut commands: Commands, buttons: Res<ButtonInput<MouseButton>>) {
  if buttons.just_pressed(MouseButton::Left) {
    log::debug!("left click");
  }
}

fn setup_camera(mut commands: Commands) {
  log::debug!("camera");
  // Camera
  commands.spawn(Camera2d);
}

fn setup_map(mut commands: Commands) {
  spawn_map(&mut commands);
}

fn setup_test(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let color = Color::srgba(0., 1., 0., 0.);

  commands.spawn((
    Mesh2d(meshes.add(Circle::new(50.0))),
    MeshMaterial2d(materials.add(color)),
    Transform::from_xyz(
      // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
      200.0,
      0.0,
      100.0,
    ),
  ));
}

fn spawn_map(commands: &mut Commands) {
  let mut rng = rand::rng();
  let (size_x, size_y) = (600, 400);
  let sprite_size = 2.;
  let color = Color::srgba(0., 0., 0., 0.);

  commands
    .spawn((
      Transform::from_xyz(
        -(size_x as f32 * sprite_size) / 2.,
        -(size_y as f32 * sprite_size) / 2.,
        0.,
      ),
      Visibility::default(),
    ))
    .with_children(|builder| {
      for y in 0..=size_y {
        for x in 0..=size_x {
          let state = ConwayCellState(rng.random_bool(1. / 3.));
          builder.spawn((
            Sprite {
              custom_size: Some(Vec2::splat(sprite_size)),
              color,
              ..default()
            },
            Transform::from_xyz(
              sprite_size * x as f32,
              sprite_size * y as f32,
              0.,
            ),
            MooreCell2d::new(IVec2::new(x, y)),
            state,
          ));
        }
      }
    });
  println!("map generated");
}
