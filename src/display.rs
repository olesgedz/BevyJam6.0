use bevy::{
  asset::{Asset, Assets, Handle},
  ecs::{
    component::Component,
    system::{Commands, Res, ResMut},
  },
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
  board_buffers::BoardBuffers, constants::{DISPLAY_FACTOR, SIZE}, shader_types::MaterialInfo,
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
