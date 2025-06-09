use bevy::{
  log,
  prelude::*,
  render::{
    Render, RenderApp, RenderSet,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::{RenderAssetUsages, RenderAssets},
    render_graph::{self, RenderGraph, RenderLabel},
    render_resource::{binding_types::texture_storage_2d, *},
    renderer::{RenderContext, RenderDevice},
    storage::{GpuShaderStorageBuffer, ShaderStorageBuffer},
    texture::GpuImage,
  },
  sprite::Material2dPlugin,
};

use crate::map_gen::*;
use crate::menu::*;
use crate::shader_types::*;
use crate::{SharedGameState, map_gen};
use crate::{
  board_buffers::BoardBuffers,
  constants::*,
  display::{DisplayBoard, DisplayMaterial, do_zoom, setup_display_board},
};

use bevy::input::common_conditions::input_just_pressed;
use bevy::render::render_resource::binding_types::{
  storage_buffer, storage_buffer_read_only, uniform_buffer,
};
use bevy::render::renderer::RenderQueue;
use bevy::window::PrimaryWindow;
use bytemuck::{Pod, Zeroable};
// use shader_types::*;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::{borrow::Cow, time::Duration};

#[derive(Resource, Clone, ExtractResource, Default)]
struct BoardChanges {
  unapplied_changes: bool,
  compute_will_run: bool,
  x: usize,
  y: usize,
  board_index: usize,
  new_cell: CellState,
}

/// This system runs to place a human cell at the
/// cursor location.
fn place_human(
  board: Single<&Transform, With<DisplayBoard>>,
  camera_q: Query<(&Camera, &GlobalTransform)>,
  window_q: Query<&Window, With<PrimaryWindow>>,
  mut board_changes: ResMut<BoardChanges>,
) {
  log::debug!("running place human");
  if let Some(screen_pos) = window_q.single().unwrap().cursor_position() {
    let (camera, camera_transform) = camera_q.single().unwrap();
    if let Ok(world_pos) =
      camera.viewport_to_world(camera_transform, screen_pos)
    {
      let world_pos = world_pos.origin.truncate();
      log::debug!("world pos");
      let transform = board.into_inner();
      let translation = transform.translation.truncate();
      let size = Vec2::new(SIZE.0 as f32, SIZE.1 as f32) * DISPLAY_FACTOR;
      let half_size = size / 2.0;

      let min = translation - half_size;
      let max = translation + half_size;

      log::debug!("world {world_pos} min {min} max {max}");
      if world_pos.x >= min.x
        && world_pos.x <= max.x
        && world_pos.y >= min.y
        && world_pos.y <= max.y
      {
        // Local position within the sprite
        let local_pos = world_pos - translation + half_size;

        // Get the image and calculate pixel coordinates
        //let image_size =
        //  Vec2::new(SIZE.0 as f32, SIZE.1 as f32) * DISPLAY_FACTOR as f32;

        let uv = (local_pos / DISPLAY_FACTOR).floor();

        log::debug!("Clicked pixel: {uv}");
        board_changes.x = uv.x as usize;
        board_changes.y = SIZE.1 as usize - (uv.y as usize);
        board_changes.new_cell = CellState {
          stored_status: 1,
          population: 100,
          ..default()
        };
        board_changes.unapplied_changes = true;
      }
    }
  }
}

#[derive(Resource)]
struct ComputeTimer(Timer);

fn setup_board_buffers(
  mut commands: Commands,
  mut compute_pass_control: ResMut<ComputePassControl>,
  mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
  let mut blank_buffer = map_gen::generate_map();
  let mut buffer = ShaderStorageBuffer::from(blank_buffer);
  buffer.buffer_description.usage =
    BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE;
  // We're skipping that using the image.
  // TODO: I'm going to need to set things up so I can fucking readback too
  // I guess as a stop gap I could write a rendering buffer but fuck that too
  let buffer0 = buffers.add(buffer.clone());
  let buffer1 = buffers.add(buffer);

  commands.insert_resource(BoardBuffers {
    board_a: buffer0,
    board_b: buffer1,
  });
  compute_pass_control.active = true;
}

fn is_compute_pass_active(
  compute_pass_control: Option<Res<ComputePassControl>>,
) -> bool {
  compute_pass_control.map(|cpc| cpc.active).unwrap_or(false)
}

pub(crate) struct ZombieComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct ZombieGameLabel;

fn tick_compute_timer(time: Res<Time>, mut timer: ResMut<ComputeTimer>) {
  timer.0.tick(time.delta());
}

// runs during the render stage
fn apply_board_changes(
  mut board_changes: ResMut<BoardChanges>,
  gpu_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
  render_queue: Res<RenderQueue>,
  board_buffers: Res<BoardBuffers>,
  timer: Res<ComputeTimer>,
) {
  // Either index works only some of the time. Since only one
  // should work, it suggests that there's some kind of problem
  // with ordering of changes?
  //
  // I bet it's because we're enqueuing it for the render graph,
  // but we don't specify when the write should happen.
  //
  // We could make it part of the render graph, or we could also
  // just make it so that this only happens when the compute
  // won't be running. Which is probably superior anyway,
  // because I don't think the GPU likes reading and writing
  // from the same thing multiple times? Actually, the
  // render graph will have tools for that.
  //
  // UPDATE: adding a flag to indicate if compute will run didn't
  // work.
  //
  // UPDATE: Oh, the problem is that this will always run before
  // the update method runs.
  //
  // UPDATE: Okay, I fixed it so the flag should be the same as the
  // compute stuff. Still broken.
  //
  // UPDATE: I fixed the bug by applying the update to both buffers.
  let compute_will_run = timer.0.just_finished();
  // let buffer_handle = if board_changes.board_index == 1 {
  //   &board_buffers.board_a
  // } else {
  //   &board_buffers.board_b
  // };
  if board_changes.unapplied_changes && compute_will_run {
    //log::debug!("Compute will run, unapplied changes are present");
  }
  if board_changes.unapplied_changes && !compute_will_run {
    log::debug!(
      "applying unapplied changes to {}, {}",
      board_changes.x,
      board_changes.y
    );
    //log::debug!("board index {}\n", board_changes.board_index);
    let buffer_a = gpu_buffers.get(&board_buffers.board_a).unwrap();
    let buffer_b = gpu_buffers.get(&board_buffers.board_b).unwrap();
    let index = (board_changes.y * SIZE.0 as usize + board_changes.x);
    let mem_location = index * std::mem::size_of::<CellState>();
    // There are types like RawBufferVec that have
    // nicer set operations, but I think they copy the
    // entire CPU buffer to the GPU, when we only want
    // to change a small part because the CPU buffer will
    // be out of date very quickly.
    render_queue.write_buffer(
      &buffer_b.buffer,
      mem_location as u64,
      bytemuck::bytes_of(&board_changes.new_cell),
    );
    render_queue.write_buffer(
      &buffer_a.buffer,
      mem_location as u64,
      bytemuck::bytes_of(&board_changes.new_cell),
    );
    board_changes.unapplied_changes = false;
  }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameSet {
  Setup,
  Active,
}

impl Plugin for ZombieComputePlugin {
  fn build(&self, app: &mut App) {
    // Extract the game of life image resource from the main world into the render world
    // for operation on by the compute shader and display on the sprite.
    //
    // This is added to the mFain world.
    let shared = app.world_mut().resource::<SharedGameState>().0.clone();

    app
      .add_plugins((
        ExtractResourcePlugin::<BoardBuffers>::default(),
        ExtractResourcePlugin::<BoardChanges>::default(),
        ExtractResourcePlugin::<ComputePassControl>::default(),
        Material2dPlugin::<DisplayMaterial>::default(),
      ))
      .init_resource::<BoardChanges>()
      .init_resource::<ComputePassControl>()
      .add_systems(
        OnEnter(GameState::Game),
        (
          setup_board_buffers,
          setup_display_board.after(setup_board_buffers),
        )
          .in_set(GameSet::Setup),
      )
      .add_systems(
        Update,
        (
          //switch_textures.run_if(in_state(GameState::Game)),
          (
            do_zoom,
            place_human.run_if(input_just_pressed(MouseButton::Left)),
          )
            .in_set(GameSet::Active)
            .after(GameSet::Setup)
            .run_if(in_state(GameState::Game))
        ),
      );
    let render_app = app
      .sub_app_mut(RenderApp)
      .add_plugins((
      ))
      .add_systems(
        Render,
        (
          prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
          apply_board_changes.in_set(RenderSet::PrepareResources),
          tick_compute_timer.before(apply_board_changes),
        ).run_if(is_compute_pass_active)
      )
      // we might be able to use an extract resource for this?
      // but this works fine
      .insert_resource(SharedGameState(shared))
      .insert_resource(ComputeTimer(Timer::new(
        Duration::from_millis(UPDATE_RATE),
        TimerMode::Repeating,
      )));
    let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
    // to turn off the compute
    // we can remove the node with remove_node
    render_graph.add_node(ZombieGameLabel, ZombieGameNode::default());
    render_graph
      .add_node_edge(ZombieGameLabel, bevy::render::graph::CameraDriverLabel);
  }

  fn finish(&self, app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app.init_resource::<ZombieGamePipeline>();
  }
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
  pipeline: Res<ZombieGamePipeline>,
  gpu_buffers: Res<RenderAssets<GpuShaderStorageBuffer>>,
  board_buffers: Res<BoardBuffers>,
  render_device: Res<RenderDevice>,
  render_queue: Res<RenderQueue>,
) {
  let mut constants_buffer = UniformBuffer::from(BoardConstants {
    width: SIZE.0 as i32,
    height: SIZE.1 as i32,
    padding0: 0,
    padding1: 0,
  });
  // I think these are defaults that are always present
  // but we'll include them anyway.
  constants_buffer.add_usages(BufferUsages::UNIFORM | BufferUsages::COPY_DST);
  constants_buffer.write_buffer(&*render_device, &*render_queue);
  let constants_binding =
    constants_buffer.binding().expect("Constants binding");

  let view_a = gpu_buffers
    .get(&board_buffers.board_a)
    .expect("board a buffer");
  let view_b = gpu_buffers
    .get(&board_buffers.board_b)
    .expect("board b buffer");
  let bind_group_0 = render_device.create_bind_group(
    None,
    &pipeline.texture_bind_group_layout,
    &BindGroupEntries::sequential((
      view_a.buffer.as_entire_binding(),
      view_b.buffer.as_entire_binding(),
      constants_binding.clone(),
    )),
  );
  let bind_group_1 = render_device.create_bind_group(
    None,
    &pipeline.texture_bind_group_layout,
    &BindGroupEntries::sequential((
      view_b.buffer.as_entire_binding(),
      view_a.buffer.as_entire_binding(),
      constants_binding,
    )),
  );
  commands.insert_resource(ZombieBoardBindGroups([bind_group_0, bind_group_1]));
}

#[derive(Resource)]
struct ZombieGamePipeline {
  texture_bind_group_layout: BindGroupLayout,
  update_pipeline: CachedComputePipelineId,
}

impl FromWorld for ZombieGamePipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();
    let texture_bind_group_layout = render_device.create_bind_group_layout(
      "GameOfLifeImages",
      &BindGroupLayoutEntries::sequential(
        ShaderStages::COMPUTE,
        (
          storage_buffer_read_only::<Vec<CellState>>(false),
          storage_buffer::<Vec<CellState>>(false),
          uniform_buffer::<BoardConstants>(false),
        ),
      ),
    );
    let shader = world.load_asset(SHADER_ASSET_PATH);
    let pipeline_cache = world.resource::<PipelineCache>();
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

    ZombieGamePipeline {
      texture_bind_group_layout,
      update_pipeline,
    }
  }
}

#[derive(Resource, ExtractResource, Clone, Default)]
struct ComputePassControl {
  active: bool,
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
    let compute_pass_control = world.resource::<ComputePassControl>();
    if !compute_pass_control.active {
      return;
    }

    let pipeline = world.resource::<ZombieGamePipeline>();
    let pipeline_cache = world.resource::<PipelineCache>();
    let timer = world.resource::<ComputeTimer>();
    self.should_run = timer.0.just_finished();

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
      if let ZombieGameState::Update(i) = self.state {
        let mut changes = world.resource_mut::<BoardChanges>();
        changes.board_index = i;
      }
    }
  }

  fn run(
    &self,
    _graph: &mut render_graph::RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), render_graph::NodeRunError> {
    if self.should_run {
      // select the pipeline based on the current state
      match self.state {
        ZombieGameState::Init => {}
        ZombieGameState::Update(index) => {
          let bind_groups = &world.resource::<ZombieBoardBindGroups>().0;
          let pipeline_cache = world.resource::<PipelineCache>();
          let pipeline = world.resource::<ZombieGamePipeline>();
          let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
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
