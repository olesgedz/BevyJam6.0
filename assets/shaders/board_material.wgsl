#import bevy_sprite::mesh2d_vertex_output::VertexOutput

#import "shaders/types.wgsl"::Cell;
#import "shaders/types.wgsl"::CellBuffer;

struct MaterialInfo {
  offset_x: i32,
  offset_y: i32,
  width: i32,
  height: i32,
};

@group(2) @binding(0) var<storage, read> input: CellBuffer;
@group(2) @binding(1) var<uniform> info: MaterialInfo;

fn load(location: vec2<i32>) -> Cell {
  let pos = location + vec2(info.offset_x, info.offset_y);
  return input.values[pos.y * info.width + location.x];
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
  let green = vec4<f32>(0., 1., 0., 1.);
  let blue = vec4<f32>(0., 0., 1., 1.);
  let pos = vec2(i32(mesh.uv.x * f32(info.width)), i32(mesh.uv.y * f32(info.height)));
  let cell = load(pos);

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
  return color;
}
