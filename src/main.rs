//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

mod constants;
mod map_gen;
mod terrain;

use bevy::{
  color::palettes::css::{GREEN, ROYAL_BLUE, SANDY_BROWN},
  dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
  log::{self, LogPlugin},
  prelude::*,
  render::{
    Render, RenderApp, RenderSet,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::{RenderAssetUsages, RenderAssets},
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{
      binding_types::{
        storage_buffer, storage_buffer_read_only, texture_storage_2d,
        uniform_buffer,
      },
      *,
    },
    renderer::{RenderContext, RenderDevice},
    storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
    texture::GpuImage,
  },
};
use bytemuck::{Pod, Zeroable};
use constants::*;
use map_gen::CellState;
use rand::Rng;
use std::{borrow::Cow, time::Duration};

fn main() {
  App::new()
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins((
      DefaultPlugins
        .set(LogPlugin {
          filter: "warn,bevy_jam6=debug".to_string(),
          ..default()
        })
        .set(WindowPlugin {
          primary_window: Some(Window {
            resolution: (
              (SIZE.0 * DISPLAY_FACTOR) as f32,
              (SIZE.1 * DISPLAY_FACTOR) as f32,
            )
              .into(),
            // uncomment for unthrottled FPS
            // present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
          }),
          ..default()
        })
        .set(ImagePlugin::default_nearest()),
      FpsOverlayPlugin {
        config: FpsOverlayConfig {
          text_config: TextFont {
            // Here we define size of our overlay
            font_size: 42.0,
            ..default()
          },
          // We can also change color of the overlay
          //text_color: OverlayColor::GREEN,
          // We can also set the refresh interval for the FPS counter
          refresh_interval: core::time::Duration::from_millis(100),
          enabled: true,
          ..default()
        },
      },
      ZombieComputePlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, switch_textures)
    .run();
}

// Switch texture to display every frame to show the one that was written to most recently.
//
// We need to switch because the GPU doesn't like writing and then reading from the same
// board multiple times in a frame.
fn switch_textures(
  buffers: Res<BoardBuffers>,
  mut sprite: Single<&mut Sprite>,
) {
  if sprite.image == buffers.image_a {
    sprite.image = buffers.image_b.clone_weak();
  } else {
    sprite.image = buffers.image_a.clone_weak();
  }
}

#[derive(Resource)]
struct ComputeTimer(Timer);

fn setup(
  mut commands: Commands,
  mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
  mut images: ResMut<Assets<Image>>,
) {
  let mut image = Image::new_fill(
    Extent3d {
      width: SIZE.0,
      height: SIZE.1,
      depth_or_array_layers: 1,
    },
    TextureDimension::D2,
    &[0, 0, 0, 255],
    TextureFormat::Rgba8Unorm,
    RenderAssetUsages::RENDER_WORLD,
  );
  image.texture_descriptor.usage = TextureUsages::COPY_DST
    | TextureUsages::STORAGE_BINDING
    | TextureUsages::TEXTURE_BINDING
    | TextureUsages::RENDER_ATTACHMENT;
  let image_handle_a = images.add(image.clone());
  let image_handle_b = images.add(image);
  let mut blank_buffer = map_gen::generate_map();
  let humans = blank_buffer
    .iter()
    .filter(|cell| cell.stored_status == 1)
    .count();
  log::debug!("human count {}", humans);
  let mut buffer = ShaderStorageBuffer::from(blank_buffer);
  buffer.buffer_description.usage =
    BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE;
  // We're skipping that using the image.
  // TODO: I'm going to need to set things up so I can fucking readback too
  // I guess as a stop gap I could write a rendering buffer but fuck that too
  let buffer0 = buffers.add(buffer.clone());
  let buffer1 = buffers.add(buffer);
  commands.spawn((
    Sprite {
      image: image_handle_a.clone(),
      custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
      ..default()
    },
    Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),
  ));

  commands.spawn(Camera2d);

  commands.insert_resource(BoardBuffers {
    board_a: buffer0,
    board_b: buffer1,
    image_a: image_handle_a,
    image_b: image_handle_b,
  });
}

/*
#[derive(Debug, Clone, Copy, Component)]
struct Tile(Vec2);

#[derive(Resource)]
struct TerrainMaterial(Handle<ColorMaterial>);

#[derive(Resource)]
struct ZombieMaterial(Handle<ColorMaterial>);

#[derive(Resource)]
struct HumanMaterial(Handle<ColorMaterial>);


fn spawn_tiles(
  mut commands: Commands,
  mut materials: ResMut<Assets<ColorMaterial>>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  let terrain_material_handle = materials.add(Color::from(SANDY_BROWN));
  let zombie_material_handle = materials.add(Color::from(GREEN));
  let human_material_handle = materials.add(Color::from(ROYAL_BLUE));

  commands.insert_resource(TerrainMaterial(terrain_material_handle));
  commands.insert_resource(ZombieMaterial(zombie_material_handle));
  commands.insert_resource(HumanMaterial(human_material_handle));
  let rect = Rectangle::from_size(Vec2::splat(1.0));
  let rect_mesh_handle = meshes.add(rect);

  for x in 0..SIZE.0 {
    for y in 0..SIZE.1 {
      commands.spawn((
        Mesh2d(rect_mesh_handle),
        MeshMaterial2d(zombie_material_handle),
        Tile(Vec2::new(x, y)),
        Transform {
          translation: Vec3(DISPLAY_FACTOR * x, DISPLAY_FACTOR * y, 0.),
          ..default()
        },
      ));
    }
  }
}

fn show_board(
  boards: Res<BoardBuffers>,
  terrain_material: Res<TerrainMaterial>,
  zombie_material: Res<ZombieMaterial>,
  human_material: Res<HumanMaterial>,
  tiles: &mut Query<(&Tile, &mut MeshMaterial2d)>,
) {
  for (tile, mut mesh) in tiles {
    /*
    let board = boards.active();
    mesh.0 = if board[tile.0.y * SIZE.0 + tile.0.x] {
      zombie_material.0
    } else { human_material.0 };
    */
  }
}
*/

struct ZombieComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct GameOfLifeLabel;

impl Plugin for ZombieComputePlugin {
  fn build(&self, app: &mut App) {
    // Extract the game of life image resource from the main world into the render world
    // for operation on by the compute shader and display on the sprite.
    app.add_plugins(ExtractResourcePlugin::<BoardBuffers>::default());
    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(
      Render,
      prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
    );

    render_app.insert_resource(ComputeTimer(Timer::new(
      Duration::from_millis(UPDATE_RATE),
      TimerMode::Repeating,
    )));

    let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
    render_graph.add_node(GameOfLifeLabel, ZombieGameNode::default());
    render_graph
      .add_node_edge(GameOfLifeLabel, bevy::render::graph::CameraDriverLabel);
  }

  fn finish(&self, app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app.init_resource::<GameOfLifePipeline>();
  }
}

#[derive(Resource, Clone, ExtractResource)]
struct BoardBuffers {
  board_a: Handle<ShaderStorageBuffer>,
  board_b: Handle<ShaderStorageBuffer>,
  image_a: Handle<Image>,
  image_b: Handle<Image>,
}

// The way the pipeline works, we give the pipeline a list
// of buffers and resources when we run it that correspond to
// certain indices. Because we want to swap the order of some
// resources, we need to store two different "bindings", each
// for a given order.
#[derive(Resource)]
struct ZombieBoardBindGroups([BindGroup; 2]);

fn prepare_bind_group(
  mut commands: Commands,
  pipeline: Res<GameOfLifePipeline>,
  gpu_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
  gpu_images: Res<RenderAssets<GpuImage>>,
  board_buffers: Res<BoardBuffers>,
  render_device: Res<RenderDevice>,
) {
  let view_a = gpu_buffers
    .get(&board_buffers.board_a)
    .expect("board a buffer");
  let view_b = gpu_buffers
    .get(&board_buffers.board_b)
    .expect("board b buffer");
  let image_a = gpu_images
    .get(&board_buffers.image_a)
    .expect("image out buffer");
  let image_b = gpu_images
    .get(&board_buffers.image_b)
    .expect("image out buffer");
  let bind_group_0 = render_device.create_bind_group(
    None,
    &pipeline.texture_bind_group_layout,
    &BindGroupEntries::sequential((
      view_a.buffer.as_entire_binding(),
      view_b.buffer.as_entire_binding(),
      &image_a.texture_view,
    )),
  );
  let bind_group_1 = render_device.create_bind_group(
    None,
    &pipeline.texture_bind_group_layout,
    &BindGroupEntries::sequential((
      view_b.buffer.as_entire_binding(),
      view_a.buffer.as_entire_binding(),
      &image_b.texture_view,
    )),
  );
  commands.insert_resource(ZombieBoardBindGroups([bind_group_0, bind_group_1]));
}

#[derive(Resource)]
struct GameOfLifePipeline {
  texture_bind_group_layout: BindGroupLayout,
  //init_pipeline: CachedComputePipelineId,
  update_pipeline: CachedComputePipelineId,
}

impl FromWorld for GameOfLifePipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();
    let texture_bind_group_layout = render_device.create_bind_group_layout(
      "GameOfLifeImages",
      &BindGroupLayoutEntries::sequential(
        ShaderStages::COMPUTE,
        (
          storage_buffer_read_only::<Vec<CellState>>(false),
          storage_buffer::<Vec<CellState>>(false),
          // see https://docs.rs/bevy/latest/src/custom_post_processing/custom_post_processing.rs.html#302-307
          //uniform_buffer::<WidthSettings>(true),
          // old
          //texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
          texture_storage_2d(
            TextureFormat::Rgba8Unorm,
            StorageTextureAccess::WriteOnly,
          ),
        ),
      ),
    );
    let shader = world.load_asset(SHADER_ASSET_PATH);
    let pipeline_cache = world.resource::<PipelineCache>();
    /*
    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: None,
        layout: vec![texture_bind_group_layout.clone()],
        push_constant_ranges: Vec::new(),
        shader: shader.clone(),
        shader_defs: vec![],
        entry_point: Cow::from("init"),
        zero_initialize_workgroup_memory: false,
    });
    */
    let update_pipeline =
      pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: None,
        layout: vec![texture_bind_group_layout.clone()],
        push_constant_ranges: Vec::new(),
        shader,
        shader_defs: vec![],
        entry_point: Cow::from("update"),
        zero_initialize_workgroup_memory: false,
      });

    GameOfLifePipeline {
      texture_bind_group_layout,
      //init_pipeline,
      update_pipeline,
    }
  }
}

enum ZombieGameState {
  Init,
  // 0 means a is active
  Update(usize),
}

struct ZombieGameNode {
  state: ZombieGameState,
  should_run: bool,
}

impl Default for ZombieGameNode {
  fn default() -> Self {
    Self {
      state: ZombieGameState::Init,
      should_run: false,
    }
  }
}

impl render_graph::Node for ZombieGameNode {
  fn update(&mut self, world: &mut World) {
    world.resource_scope(|world, mut timer: Mut<ComputeTimer>| {
      let time = world.resource::<Time>();
      let should_run = timer.0.tick(time.delta());
      self.should_run = timer.0.just_finished();
    });
    let pipeline = world.resource::<GameOfLifePipeline>();
    let pipeline_cache = world.resource::<PipelineCache>();

    if self.should_run {
      // if the corresponding pipeline has loaded, transition to the next stage
      match self.state {
        ZombieGameState::Init => {
          match pipeline_cache
            .get_compute_pipeline_state(pipeline.update_pipeline)
          {
            CachedPipelineState::Ok(_) => {
              self.state = ZombieGameState::Update(1);
            }
            CachedPipelineState::Err(err) => {
              panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
            }
            _ => {}
          }
        }
        ZombieGameState::Update(0) => {
          self.state = ZombieGameState::Update(1);
        }
        ZombieGameState::Update(1) => {
          self.state = ZombieGameState::Update(0);
        }
        ZombieGameState::Update(_) => unreachable!(),
      }
    }
  }

  fn run(
    &self,
    _graph: &mut render_graph::RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), render_graph::NodeRunError> {
    let bind_groups = &world.resource::<ZombieBoardBindGroups>().0;
    let pipeline_cache = world.resource::<PipelineCache>();
    let pipeline = world.resource::<GameOfLifePipeline>();

    if self.should_run {
      let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor::default());

      // select the pipeline based on the current state
      match self.state {
        ZombieGameState::Init => {}
        ZombieGameState::Update(index) => {
          let update_pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.update_pipeline)
            .unwrap();
          pass.set_bind_group(0, &bind_groups[index], &[]);
          pass.set_pipeline(update_pipeline);
          pass.dispatch_workgroups(
            SIZE.0 / WORKGROUP_SIZE,
            SIZE.1 / WORKGROUP_SIZE,
            1,
          );
        }
      }
    }

    Ok(())
  }
}
