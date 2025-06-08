//! Types that are used to communicate with the shader.
use bytemuck::{Pod, Zeroable};
use bevy::render::render_resource::ShaderType;

#[derive(Debug, Default, Clone, Copy, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct BoardConstants {
  pub width: i32,
  pub height: i32,
  // pad to 16 bytes
  pub padding0: i32,
  pub padding1: i32,
}

#[derive(Debug, Default, Clone, Copy, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct CellState {
  pub neighbors_count: i32,
  pub edge_distance: i32,
  pub altitude: i32,
  pub temperature: i32,
  pub population: i32,
  // we need an extra 4 bytes for 16 byte alignment
  // anyway, and it's much easier in the shader
  // to have the direction as a delta.
  pub direction_x: i32,
  pub direction_y: i32,
  pub second_direction_x: i32,
  pub second_direction_y: i32,
  pub smell_human: i32,
  pub smell_zombie: i32,

  // for convenient alignment
  // 0 is empty, 1 is human, 2 is zombie
  pub stored_status: u32,
}
