use bevy::{
  prelude::*,
  asset::Handle,
  ecs::resource::Resource,
  image::Image,
  render::{extract_resource::ExtractResource, storage::ShaderStorageBuffer},
};

#[derive(Resource, Clone, ExtractResource)]
pub struct BoardBuffers {
  pub board_a: Handle<ShaderStorageBuffer>,
  pub board_b: Handle<ShaderStorageBuffer>,
  pub image_a: Handle<Image>,
  pub image_b: Handle<Image>,
}
