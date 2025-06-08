// The shader reads the previous frame's state from the `input` texture, and writes the new state of
// each pixel to the `output` texture. The textures are flipped each step to progress the
// simulation.
// Two textures are needed for the game of life as each pixel of step N depends on the state of its
// neighbors at step N-1.

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

struct Constants {
  width: i32,
  height: i32,
  padding0: i32,
  padding1: i32,
};

@group(0) @binding(0) var<storage, read> input: CellBuffer;

// read_write is required even if only writing
@group(0) @binding(1) var<storage, read_write> output: CellBuffer;

// output image
@group(0) @binding(2) var image_out: texture_storage_2d<rgba8unorm, write>;


// Couldn't figure out overrides
@group(0) @binding(3) var<uniform> constants: Constants;

// this is an overridable constant that can be changed when we
// create the shader pipeline
//
// I can't figure out how to do this easily in bevy, so we'll use uniform
// instead.
//override width = 200;
//override height = 200;

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
  return input.values[location.y * constants.width + location.x];
}

fn store(location: vec2<i32>, cell: Cell) {
  output.values[location.y * constants.width + location.x] = cell;
  let green = vec4<f32>(0., 1., 0., 1.);
  let blue = vec4<f32>(0., 0., 1., 1.);


  var color = vec4(0.);

  if cell.status == 1 {
      color = blue;
  } else if cell.status == 2 {
      color = green;
  } else {

    var max_zombie_smell: i32 = 5000;
    var max_human_smell: i32 = 5000;
    // darn inefficient, as it iterates over all cells per stored cell, so N^N
    // for (var i = 0; i < i32(arrayLength(&input.values)); i++) {
    //   max_human_smell = max(max_human_smell, input.values[i].human_smell);
    //   max_zombie_smell = max(max_zombie_smell, input.values[i].zombie_smell);
    // }

    let human_smell_rel = max(min(f32(cell.human_smell) / f32(max_human_smell), 0.5), 0.);
    let human_smell_color = vec4<f32>(0., 0., human_smell_rel, 0.5);

    let zombie_smell_rel = max(min(f32(cell.zombie_smell) / f32(max_zombie_smell), 0.5), 0.);
    let zombie_smell_color = vec4<f32>(0., zombie_smell_rel, 0., 0.5);

    let smell_color = human_smell_color + zombie_smell_color;
    color = smell_color;
  }

  textureStore(image_out, location, color);
}

struct Population {
  humans: u32,
  zombies: u32
};

const humans_soft_cap: i32 = 600;
const zombies_soft_cap: i32 = 1200;
const smell_decay: f32 = 1.0005;

fn get_dir_delta(cell: Cell) -> vec2<i32> {
  return vec2<i32>(cell.direction_x, cell.direction_y);
}

fn get_second_dir_delta(cell: Cell) -> vec2<i32> {
  return vec2<i32>(cell.second_direction_x, cell.second_direction_y);
}

fn is_pos_valid(pos: vec2<i32>) -> bool {
  return pos.x >= 0 && pos.x < constants.width && pos.y >= 0 && pos.y < constants.height;
  //return all(pos >= vec2(0,0)) && all(pos < vec2(constants.height, constants.width));
}

// x is humans, y is zombies, z is human smell, w is zombie smell
fn incoming_from_delta(pos: vec2<i32>, delta_x: i32, delta_y: i32) -> vec4<i32> {
  let delta = vec2(delta_x, delta_y);
  let neighbor = pos + delta;
  if !is_pos_valid(neighbor) {
    return vec4(0, 0, 0, 0);
  }
  let cell = load(neighbor);
  var pops = vec2(0);
  if all(get_dir_delta(cell) == (delta * -1)) || all(get_second_dir_delta(cell) == (delta * -1)) {
    let is_splitting = cell.second_direction_x != 0 || cell.second_direction_y != 0;

    var in_population = 0;
    if is_splitting {
        in_population = cell.population / 2;
    } else {
        in_population = cell.population;
    }

    if cell.status == 1 {
      pops = vec2(in_population, 0);
    } else if cell.status == 2 {
      pops = vec2(0, in_population);
    }
  }
  return vec4(pops, cell.human_smell / (cell.neighbors_count + 1), cell.zombie_smell / (cell.neighbors_count + 1));
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

fn most_neighboring_zombie_smell(pos: vec2<i32>) -> i32 {
  var max_smell = 0;
  for (var i = 0; i < 8; i++) {
    let neighbor = pos + directions[i];
    if is_pos_valid(neighbor) {
      let cell = load(neighbor);
      if cell.zombie_smell > max_smell {
        max_smell = cell.zombie_smell;
      }
    }
  }
  return max_smell;
}

fn zombie_preferred_neighbor(pos: vec2<i32>, except_direction: vec2<i32>) -> vec2<i32> {
  var pursue_dir = vec2<i32>();
  var roam_dir = vec2<i32>();
  var max_human_smell = 0;
  var min_zombie_smell = 10000000;
  var in_pursuit = false;

  for (var i=0; i<8; i++) {
    let neighbor = pos + directions[i];
    if is_pos_valid(neighbor) && all(directions[i] != except_direction) {
        let cell = load(neighbor);
        if cell.human_smell > max_human_smell {
            pursue_dir = directions[i];
            max_human_smell = cell.human_smell;
            in_pursuit = true;
          } else if !in_pursuit && cell.zombie_smell < min_zombie_smell {
            roam_dir = directions[i];
            min_zombie_smell = cell.zombie_smell;
          }
      }
  }
  if in_pursuit {
      return pursue_dir;
    } else {
    return roam_dir;
  }
}

// action threshold, things need to either be getting too bad, or there's a way to get it way better.
// E.g., zombies coming (getting bad); there's a good spot (higher altitude, less populated) (could get way better).
const laziness: i32 = 200;

// Helper function to find the direction with the least zombie smell;        preference, direction
fn human_preferred_neighbor(pos: vec2<i32>, except_direction: vec2<i32>) -> vec2<i32> {
  var max_dir = vec2<i32>();
  var max_preference: i32 = -100000000;
  var min_preference: i32 = 100000000;

  for (var i = 0; i < 8; i++) {
    let neighbor = pos + directions[i];
    if is_pos_valid(neighbor) && all(directions[i] != except_direction) {
      let cell = load(neighbor);

      // humans don't like to be cornered (who does?)
      var not_too_close_to_edge_pref: i32 = 0;
      if cell.edge_distance < 10 {
          not_too_close_to_edge_pref = (10 - cell.edge_distance) * -50;
      }

      let preference = cell.zombie_smell * -40
        + cell.altitude * 10
        // + cell.temperature * 20
        + cell.population * -20
        // + cell.human_smell * -10 // who likes living in a crowded space? (ok, in a zombie apoc it actually may be wise)
        + not_too_close_to_edge_pref;

      max_preference = max(max_preference, preference);
      min_preference = min(min_preference, preference);

      if preference >= max_preference {
          max_dir = directions[i];
        }
    }
  }

  let self_cell = load(pos);

  if self_cell.population >= humans_soft_cap || min_preference <= laziness * -1 || max_preference >= laziness {
      return max_dir;
    } else {
    return vec2(0);
  }

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

  var humans_kia = 0;
  var zombies_kia = 0;

  if humans == zombies {
    new_cell.population = 0;
    new_cell.status = 0;
    humans_kia = humans;
    zombies_kia = zombies;
  } else if humans > zombies * 6 {
    new_cell.population = humans - zombies;
    new_cell.status = 1;
    new_human_smell = new_cell.population * 3;
    humans_kia = zombies;
    zombies_kia = zombies;
  } else {
    // To represent infection
    new_cell.population = zombies + humans;
    new_cell.status = 2;
    new_zombie_smell = new_cell.population;
    humans_kia = humans;
    // zombies_kia = humans;
  }

  new_human_smell += humans_kia * 5; // blood has been spilled
  new_zombie_smell += zombies_kia * 5; // rotten flesh has been slashed

  // Update smell
  new_cell.human_smell = i32(f32(cell.human_smell / 9 + neighbor_human_smell) / smell_decay) + new_human_smell;
  new_cell.zombie_smell = i32( f32(cell.zombie_smell / 9 + neighbor_zombie_smell) / smell_decay) + new_zombie_smell;

  // Movement logic for humans
  if cell.status == 1 {
    //if zombies > humans {

      let preferred_dir = human_preferred_neighbor(pos, vec2(0));

      new_cell.direction_x = preferred_dir.x;
      new_cell.direction_y = preferred_dir.y;

      // Humans group grew too large, they prefer to split up
      if cell.population > humans_soft_cap {
          let second_preferred_dir = human_preferred_neighbor(pos, preferred_dir);
          new_cell.second_direction_x = second_preferred_dir.x;
          new_cell.second_direction_y = second_preferred_dir.y;
      } else {
        new_cell.second_direction_x = 0;
        new_cell.second_direction_y = 0;
      }


    /*} else {
      // Stand ground
      new_cell.direction_x = 0;
      new_cell.direction_y = 0;
    }*/

      new_cell.population = humans + i32(ceil(f32(humans) / 100)); // Birth / reproduction
  } else if cell.status == 2 {
    // Movement logic for zombies (unchanged)
    let dir = zombie_preferred_neighbor(pos, vec2(0));
    new_cell.direction_x = dir.x;
    new_cell.direction_y = dir.y;

    if cell.population >= zombies_soft_cap {
        let second_dir = zombie_preferred_neighbor(pos, dir);
        new_cell.second_direction_x = second_dir.x;
        new_cell.second_direction_y = second_dir.y;
      } else {
      new_cell.second_direction_x = 0;
      new_cell.second_direction_y = 0;
    }
  }

  return new_cell;
}

fn root(num: f32, n: i32) -> f32 {
  var out_num = num;
  for (var i = 0; i < n; i++) {
    out_num = sqrt(out_num);
  }
  return out_num;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

  let new_cell = calculate_new_cell(location);

  store(location, new_cell);
}
