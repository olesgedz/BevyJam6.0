// The shader reads the previous frame's state from the `input` texture, and writes the new state of
// each pixel to the `output` texture. The textures are flipped each step to progress the
// simulation.
// Two textures are needed for the game of life as each pixel of step N depends on the state of its
// neighbors at step N-1.

@group(0) @binding(0) var<storage, read> input: array<u32>;

@group(0) @binding(1) var<storage, write> output: array<u32>;

// output image
@group(0) @binding(2) var output: texture_storage_2d<r32float, write>;

// this is an overridable constant that can be changed when we
// create the shader pipeline
override width = 200;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    state = state ^ (state >> 16u);
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

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

fn load(location: vec2<i32>) -> bool {
  return input[location.y * width + location.x] > 0;
}
fn store(location: vec2<i32>, value: bool) {
  output[location.y * width + location.x] = i32(value);
  let color = vec4<f32>(f32(alive));

  textureStore(output, location, color);
}

fn is_alive(location: vec2<i32>, offset_x: i32, offset_y: i32) -> i32 {
    let value: bool = load(location + vec2<i32>(offset_x, offset_y));
    return i32(value);
}

fn count_alive(location: vec2<i32>) -> i32 {
    return is_alive(location, -1, -1) +
           is_alive(location, -1,  0) +
           is_alive(location, -1,  1) +
           is_alive(location,  0, -1) +
           is_alive(location,  0,  1) +
           is_alive(location,  1, -1) +
           is_alive(location,  1,  0) +
           is_alive(location,  1,  1);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let n_alive = count_alive(location);

    var alive: bool;
    if (n_alive == 3) {
        alive = true;
    } else if (n_alive == 2) {
        let currently_alive = is_alive(location, 0, 0);
        alive = bool(currently_alive);
    } else {
        alive = false;
    }

    store(location, alive);
}
