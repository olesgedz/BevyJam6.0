mod menu;
mod game;

use bevy::{
    log::{self, LogPlugin},
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
    },
};

// use bevy::render::renderer::*;
use std::borrow::Cow;
use menu::menu::*;
use menu::*;
use game::*;

fn main() {
    log::debug!("START");
    App::new()
        .add_plugins((
            DefaultPlugins
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
                .set(LogPlugin {
                    filter: "info,wgpu_core=warn,wgpu_hal=warn,bevy_jam6=debug".into(),
                    level: bevy::log::Level::DEBUG,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(DisplayQuality::Medium)
        .insert_resource(Volume(7))
        // Declare the game state, whose starting value is determined by the `Default` trait
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_plugins((splash::splash_plugin, menu_plugin, GameOfLifeComputePlugin))
        .run();
}

fn setup(mut commands: Commands) {
    log::debug!("SETUP");

    commands.spawn(Camera2d);
}