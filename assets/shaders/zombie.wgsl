// The shader reads the previous frame's state from the `input` texture, and writes the new state of
// each pixel to the `output` texture. The textures are flipped each step to progress the
// simulation.
// Two textures are needed for the game of life as each pixel of step N depends on the state of its
// neighbors at step N-1.

struct Cell {
  altitude: i32,
  temperature: i32,
  population: i32,
  // we need an extra 4 bytes for 16 byte alignment
  // anyway, and it's much easier in the shader
  // to have the direction as a delta.
  // 0,0 means we didn't move
  direction_x: i32,
  direction_y: i32,
  human_smell: i32,
  zombie_smell: i32,

  // for convenient alignment
  // 0 is empty, 1 is human, 2 is zombie
  status: u32,
};

struct CellBuffer {
  values: array<Cell>,
};

@group(0) @binding(0) var<storage, read> input: CellBuffer;

// read_write is required even if only writing
@group(0) @binding(1) var<storage, read_write> output: CellBuffer;

// output image
@group(0) @binding(2) var image_out: texture_storage_2d<rgba8unorm, write>;

// this is an overridable constant that can be changed when we
// create the shader pipeline
override width = 200;

// @compute @workgroup_size(8, 8, 1)
// fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
//     let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
// 
//     let randomNumber = randomFloat((invocation_id.y << 16u) | invocation_id.x);
//     let alive = randomNumber > 0.9;
//     let color = vec4<f32>(f32(alive));
// 
//     textureStore(output, location, color);
// }

// we're representing locations with i32 so we can add them
// to deltas.

fn load(location: vec2<i32>) -> Cell {
  return input.values[location.y * width + location.x];
}

fn store(location: vec2<i32>, value: Cell) {
  output.values[location.y * width + location.x] = value;
  let green = vec4<f32>(0., 1., 0., 1.);
  let blue = vec4<f32>(0., 0., 1., 1.);

  if value.status == 1 {
    textureStore(image_out, location, blue);
  } else if value.status == 2 {
    textureStore(image_out, location, green);
  } else {
    textureStore(image_out, location, vec4(0.));
  }
}

struct Population {
  humans: u32,
  zombies: u32
};

fn get_dir_delta(cell: Cell) -> vec2<i32> {
  return vec2<i32>(cell.direction_x, cell.direction_y);
}

// x is humans, y is zombies, z is human smell, w is zombie smell
fn incoming_from_delta(pos: vec2<i32>, delta_x: i32, delta_y: i32) -> vec4<i32> {
  let delta = vec2(delta_x, delta_y);
  let neighbor = pos + delta;
  let cell = load(neighbor);
  var pops = vec2(0);
  if all(get_dir_delta(cell) == (delta * -1)) {
    if cell.status == 1 {
      pops = vec2(cell.population, 0);
    } else if cell.status == 2 {
      pops = vec2(0, cell.population);
    }
  }
  return vec4(pops, cell.human_smell, cell.zombie_smell);
}

// x is humans, y is zombies, z is human smell, w is zombie smell
fn incoming_to(location: vec2<i32>) -> vec4<i32> {
    return incoming_from_delta(location, -1, -1) +
           incoming_from_delta(location, -1,  0) +
           incoming_from_delta(location, -1,  1) +
           incoming_from_delta(location,  0, -1) +
           incoming_from_delta(location,  0,  1) +
           incoming_from_delta(location,  1, -1) +
           incoming_from_delta(location,  1,  0) +
           incoming_from_delta(location,  1,  1);
}

// x is humans, y is zombies, z is human smell, w is zombie smell
fn total_in(pos: vec2<i32>, cell: Cell) -> vec4<i32> {
  var local_humans = 0;
  var local_zombies = 0;
  if all(get_dir_delta(cell) == vec2(0,0)) {
    if cell.status == 1 {
      local_humans = cell.population;
    }
    if cell.status == 2 {
      local_zombies = cell.population;
    }
  }
  return incoming_to(pos) + vec4(local_humans, local_zombies, 0, 0);
}

const directions = array<vec2<i32>,8>(
  vec2(-1, -1),
  vec2(-1,  0),
  vec2(-1,  1),
  vec2( 0, -1),
  vec2( 0,  1),
  vec2( 1, -1),
  vec2( 1,  0),
  vec2( 1,  1),
);

fn neighbor_with_most_human_smell(pos: vec2<i32>) -> vec2<i32> {
  var max_dir = vec2<i32>();
  var max_smell = 0;
  for (var i=0; i<8; i++) {
    let cell = load(pos + directions[i]);
    if cell.human_smell > max_smell {
      max_dir = directions[i];
      max_smell = cell.human_smell;
    }
  }
  return max_dir;
}

// Helper function to find the direction with the least zombie smell
fn neighbor_with_least_zombie_smell(pos: vec2<i32>) -> vec2<i32> {
  var min_dir = vec2<i32>();
  var min_smell = i32(1 << 30); // A large number to initialize
  for (var i = 0; i < 8; i++) {
    let cell = load(pos + directions[i]);
    if cell.zombie_smell < min_smell {
      min_dir = directions[i];
      min_smell = cell.zombie_smell;
    }
  }
  return min_dir;
}

// Updated calculate_new_cell function
fn calculate_new_cell(pos: vec2<i32>) -> Cell {
  let cell = load(pos);
  var new_cell = cell;
  let totals = total_in(pos, cell);
  let humans = totals.x;
  let zombies = totals.y;
  let neighbor_human_smell = totals.z;
  let neighbor_zombie_smell = totals.w;
  var new_human_smell = 0;
  var new_zombie_smell = 0;

  // Simplified rules for population changes
  if humans == zombies {
    new_cell.population = 0;
    new_cell.status = 0;
  } else if humans > zombies {
    new_cell.population = humans - zombies;
    new_cell.status = 1;
    new_human_smell = new_cell.population;
  } else {
    // To represent infection
    new_cell.population = zombies - humans + humans / 3;
    new_cell.status = 2;
    new_zombie_smell = new_cell.population;
  }

  // Update smell
  new_cell.human_smell = ((neighbor_human_smell / 8) + new_human_smell) / 2;
  new_cell.zombie_smell = ((neighbor_zombie_smell / 8) + new_zombie_smell) / 2;

  // Movement logic for humans
  if cell.status == 1 {
    //if zombies > humans {
      // Move to the cell with the least zombie smell
      let dir = neighbor_with_least_zombie_smell(pos);
      new_cell.direction_x = dir.x;
      new_cell.direction_y = dir.y;
    /*} else {
      // Stand ground
      new_cell.direction_x = 0;
      new_cell.direction_y = 0;
    }*/
  } else if cell.status == 2 {
    // Movement logic for zombies (unchanged)
    let dir = neighbor_with_most_human_smell(pos);
    new_cell.direction_x = dir.x;
    new_cell.direction_y = dir.y;
  }

  return new_cell;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

  let new_cell = calculate_new_cell(location);

  store(location, new_cell);
}
