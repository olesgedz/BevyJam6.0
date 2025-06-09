use bevy::{
  asset::{Asset, Assets, Handle},
  ecs::{
    component::Component,
    system::{Commands, Res, ResMut},
  },
  input::mouse::{MouseScrollUnit, MouseWheel},
  log,
  math::primitives::Rectangle,
  pbr::Material,
  prelude::*,
  reflect::TypePath,
  render::{
    mesh::{Mesh, Mesh2d},
    render_resource::{AsBindGroup, ShaderRef},
    storage::ShaderStorageBuffer,
  },
  sprite::{Material2d, MeshMaterial2d},
};

use crate::{
  board_buffers::BoardBuffers,
  constants::{DISPLAY_FACTOR, SIZE},
  shader_types::MaterialInfo,
};

const SHADER_ASSET_PATH: &str = "shaders/board_material.wgsl";

#[derive(Component, Copy, Clone)]
pub struct DisplayBoard;

// this material automatically displays from the buffer
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DisplayMaterial {
  #[storage(0, read_only)]
  display_board: Handle<ShaderStorageBuffer>,
  // only the first two matter; rest are padding
  #[uniform(1)]
  info: MaterialInfo,
}

impl Material2d for DisplayMaterial {
  //fn alpha_mode(&self) -> AlphaMode2d {
  //    AlphaMode2d::Mask(0.5)
  //}

  fn fragment_shader() -> ShaderRef {
    SHADER_ASSET_PATH.into()
  }
}

#[derive(Resource, Clone)]
pub struct DisplayMatRes(Handle<DisplayMaterial>);

pub fn zoom_out(
  display_mat: Res<DisplayMatRes>,
  mut materials: ResMut<Assets<DisplayMaterial>>,
) {
  let mut mat = materials.get_mut(&display_mat.0).unwrap();
  mat.info.zoom_factor *= 0.9;
}

// I can use system sets to order things maybe?
//#[derive(Default)]
//pub struct DisplayBoardPlugin;
//
//impl Plugin for DisplayBoardPlugin {
//  fn build(&self, app: &mut App) {
//
//}
//}

// taken from: https://github.com/johanhelsing/bevy_pancam/blob/2a9b686ca4254369feb94f8c7dae9a64b1fc3acd/src/lib.rs#L221C1-L232C2
/// Consumes `MouseWheel` event reader and calculates a single scalar,
/// representing positive or negative scroll offset.
fn scroll_offset_from_events(
  mut scroll_events: EventReader<MouseWheel>,
) -> f32 {
  let pixels_per_line = 100.; // Maybe make configurable?
  scroll_events
    .read()
    .map(|ev| match ev.unit {
      MouseScrollUnit::Pixel => ev.y,
      MouseScrollUnit::Line => ev.y * pixels_per_line,
    })
    .sum::<f32>()
}

pub fn do_zoom(
  scroll_events: EventReader<MouseWheel>,
  display_mat: Res<DisplayMatRes>,
  mut materials: ResMut<Assets<DisplayMaterial>>,
) {
  let mut mat = materials.get_mut(&display_mat.0).unwrap();
  const ZOOM_SENSITIVITY: f32 = 0.001;

  let scroll_offset = scroll_offset_from_events(scroll_events);
  if scroll_offset == 0. {
    return;
  }
  let zoom_mult = 1. + scroll_offset * ZOOM_SENSITIVITY * -1.;
  mat.info.zoom_factor = (mat.info.zoom_factor * zoom_mult).clamp(0.00001, 1.0);
}

pub fn setup_display_board(
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<DisplayMaterial>>,
  board_buffers: Res<BoardBuffers>,
  mut commands: Commands,
) {
  let handle = materials.add(DisplayMaterial {
    display_board: board_buffers.board_a.clone(),
    // this will eventually be the offset of our
    // view into the board.
    //
    // I'm not sure how scale will interact.
    info: MaterialInfo {
      offset_x: 0,
      offset_y: 0,
      width: SIZE.0 as i32,
      height: SIZE.1 as i32,
      zoom_factor: 1.,
      buffer_index: 0,
      ..default()
    },
  });
  let id = commands
    .spawn((
      DisplayBoard,
      Mesh2d(meshes.add(Rectangle::new(SIZE.0 as f32, SIZE.1 as f32))),
      MeshMaterial2d(handle.clone()),
      Transform::from_scale(Vec3::splat(DISPLAY_FACTOR)),
    ))
    .id();
  commands.insert_resource(DisplayMatRes(handle));
  log::debug!("Spawned entity {}", id);
}

