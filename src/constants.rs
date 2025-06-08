

/// This example uses a shader source file from the assets subdirectory
pub const SHADER_ASSET_PATH: &str = "shaders/zombie.wgsl";

pub const DISPLAY_FACTOR: f32 = 1. / 2.;
pub const SIZE: (u32, u32) = (1600, 1600);
// big
//pub const DISPLAY_FACTOR: f32 = 1. / 8.;
//pub const SIZE: (u32, u32) = (1600 * 4, 1600 * 4);
pub const WORKGROUP_SIZE: u32 = 8;
pub const BUFFER_LEN: usize = (SIZE.0 * SIZE.1) as usize;

/// How many milliseconds should pass between updates.
pub const UPDATE_RATE: u64 = 10;
