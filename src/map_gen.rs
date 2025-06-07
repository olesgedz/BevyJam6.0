use rand::{self, Rng};
use bytemuck::{Pod, Zeroable};
use bevy::render::render_resource::ShaderType;

use crate::constants::*;
use crate::terrain;

// 0 is empty, 1 is human, 2 is zombie
#[derive(Debug, Default, Clone)]
#[repr(C)]
enum CellStatus {
  #[default]
  Empty, Human, Zombie
}

#[derive(Debug, Default, Clone, Copy, ShaderType, Pod, Zeroable)]
#[repr(C)]
pub struct CellState {
  pub altitude: i32,
  pub temperature: i32,
  pub population: i32,
  // we need an extra 4 bytes for 16 byte alignment
  // anyway, and it's much easier in the shader
  // to have the direction as a delta.
  pub direction_x: i32,
  pub direction_y: i32,
  pub smell_human: i32,
  pub smell_zombie: i32,

  // for convenient alignment
// 0 is empty, 1 is human, 2 is zombie
  pub stored_status: u32,
}

pub fn generate_map() -> Vec<CellState> {
  let terrain =
    terrain::TerrainGenerator::new(42).generate(SIZE.0 as usize, SIZE.1 as usize, 5, 100.0);
  let mut rng = rand::rng();
  let mut map = vec![CellState::default(); BUFFER_LEN];
  map.iter_mut().enumerate().for_each(|(i,cell)| {
    let x = i % SIZE.0 as usize;
    let y = i / SIZE.0 as usize;
    cell.altitude = terrain[y][x].altitude as i32;
    cell.temperature = terrain[y][x].temperature as i32;

    // Temporary, randomly assign cells as human, zombie, empty, and with population
    let random_state: u8 = rng.random_range(0..3); // Randomly choose between 0-3
    cell.stored_status = match random_state {
      0 => 0, // Empty
      1 => 1, // Zombie
      2 => 2, // Human
      _ => 0, // Default to empty
    };
      // If human, give a big population. If zombie, a small one.
      cell.population = match cell.stored_status {
        // Humans have a population between 50-150
        2 => rng.random_range(50..150),
        // Zombies have a population between 1-10
        1 => rng.random_range(1..10),
// Empty cells have no population
        _ => 0
      };
  });
  map
}
