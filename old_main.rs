
//

// runs during the render stage
/*
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
    log::debug!("applying unapplied changes to {}, {}", board_changes.x, board_changes.y);
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
*/

/*
impl Plugin for ZombieComputePlugin {
  fn build(&self, app: &mut App) {
    // Extract the game of life image resource from the main world into the render world
    // for operation on by the compute shader and display on the sprite.
    //
    // This is added to the main world.
    app.add_plugins((
      ExtractResourcePlugin::<BoardBuffers>::default(),
      ExtractResourcePlugin::<BoardChanges>::default(),
    ));
    app.init_resource::<BoardChanges>();
    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_systems(
      Render,
      prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
    );
    render_app.add_systems(
      Render,
      apply_board_changes.in_set(RenderSet::PrepareResources),
    );

    render_app.add_systems(
      Render,
      tick_compute_timer.before(apply_board_changes),
    );

    render_app.insert_resource(ComputeTimer(Timer::new(
      Duration::from_millis(UPDATE_RATE),
      TimerMode::Repeating,
    )));

    let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
    render_graph.add_node(ZombieGameLabel, ZombieGameNode::default());
    render_graph
      .add_node_edge(ZombieGameLabel, bevy::render::graph::CameraDriverLabel);
  }

  fn finish(&self, app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app.init_resource::<GameOfLifePipeline>();
  }
}
*/
