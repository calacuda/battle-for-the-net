use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    light::DirectionalLightShadowMap,
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_dioxus_sync::{panels::DioxusPanel, plugins::DioxusPlugin};

use crate::{backend::BasePlugin, frontend::AppUi};

pub mod backend;
pub mod frontend;

fn main() {
    let filter = format!(
        "info,{}=trace,bevy_dioxus_hooks::query::command=error,wgpu_hal=off",
        env!("CARGO_PKG_NAME").replace("-", "_")
    );
    let level = Level::INFO;

    let default_plugins = DefaultPlugins.set(LogPlugin {
        // Set the default log level for everything
        level,
        // and use a filter string for fine-grained control
        filter: filter.clone(),
        ..default()
    });

    #[cfg(feature = "headless_ci")]
    let default_plugins = default_plugins
        .disable::<bevy::window::WindowPlugin>()
        .disable::<bevy::render::RenderPlugin>();

    // let (idle_tx, idle_rx) = crossbeam::channel::unbounded();
    // let (speed_tx, speed_rx) = crossbeam::channel::unbounded();

    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(ClearColor(Color::linear_rgb(0.1, 0.1, 0.1)))
        .add_plugins((default_plugins, FrameTimeDiagnosticsPlugin::default()))
        // .add_plugins(FpsTrackingPlugin)
        // .add_plugins(SpherePlugin)
        .add_plugins(DioxusPlugin {
            bevy_info_refresh_fps: 30,
            main_window_ui: Some(DioxusPanel::new(AppUi {
                // idle_time: idle_rx,
                // automation_speed: speed_rx,
            })),
        })
        .add_plugins(BasePlugin)
        // .add_plugins(IdleTimePlugin { idle_tx, speed_tx })
        // .add_plugins(PlayerPlugin)
        // logs log level and filters
        .add_systems(Startup, move || {
            info!("default log level is: {level}");
            info!("default log filter: \"{filter}\"");
        })
        .run();
}
