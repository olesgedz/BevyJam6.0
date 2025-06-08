#import bevy_sprite::mesh2d_vertex_output::VertexOutput

#import "shaders/types.wgsl"::Cell;
#import "shaders/types.wgsl"::CellBuffer;

struct MaterialInfo {
  offset_x: i32,
  offset_y: i32,
  width: i32,
  height: i32,
  // not used yet
  buffer_index: i32,
  // tells us how zoomed in we are
  zoom_factor: f32,
  padding0: i32,
  padding1: i32,
};

@group(2) @binding(0) var<storage, read> input: CellBuffer;
@group(2) @binding(1) var<uniform> info: MaterialInfo;

fn load(mesh: VertexOutput) -> Cell {
  let offset = vec2(f32(info.offset_x), f32(info.offset_y));
  let dim = vec2(f32(info.width), f32(info.height));
  let percent_rel_to_center = (mesh.uv - .5) * 2.;
  let percent_with_zoom = percent_rel_to_center * info.zoom_factor;
  let center = (dim / 2.);
  // relative to a central origin
  let local_coord = center * percent_with_zoom;
  let absolute = offset + center + local_coord;
  let location = vec2( i32(absolute.x), i32(absolute.y));
  let pos = location + vec2(info.offset_x, info.offset_y);
  return input.values[pos.y * info.width + location.x];
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
  let green = vec4<f32>(0., 1., 0., 1.);
  let blue = vec4<f32>(0., 0., 1., 1.);
  let cell = load(mesh);

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
