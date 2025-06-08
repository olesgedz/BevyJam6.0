use rand::{self, Rng};

use crate::constants::*;
use crate::terrain;

use crate::shader_types::*;

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
