
struct Cell {
  neighbors_count: i32,
  edge_distance: i32,
  altitude: i32,
  temperature: i32,
  population: i32,
  // we need an extra 4 bytes for 16 byte alignment
  // anyway, and it's much easier in the shader
  // to have the direction as a delta.
  // 0,0 means we didn't move
  direction_x: i32,
  direction_y: i32,
  second_direction_x: i32,
  second_direction_y: i32,
  human_smell: i32,
  zombie_smell: i32,

  // for convenient alignment
  // 0 is empty, 1 is human, 2 is zombie
  status: u32,
};

struct CellBuffer {
  values: array<Cell>,
};
