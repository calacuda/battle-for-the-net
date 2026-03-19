use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    light::DirectionalLightShadowMap,
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_skein::SkeinPlugin;
// use bevy_spritefusion::prelude::*;
use iyes_progress::{Progress, ProgressPlugin, ProgressReturningSystem, ProgressTracker};

use crate::base::BasePlugin;

pub mod base;
pub mod tiled;

#[derive(AssetCollection, Resource)]
struct SpriteTiles {
    // #[asset(path = "../assets/tile-sets/single-png/", collection(mapped, typed), image(sampler(filter = nearest)))]
    // floor: HashMap<AssetFileStem, Handle<Image>>,
    // #[asset(path = "../assets/sprites/single-png/", collection(mapped, typed), image(sampler(filter = nearest)))]
    // sprites: HashMap<AssetFileStem, Handle<Image>>,
    #[asset(texture_atlas_layout(
        tile_size_x = 16,
        tile_size_y = 16,
        padding_x = 1,
        padding_y = 1,
        rows = 12,
        columns = 54
    ))]
    pub sprite_sheet: Handle<TextureAtlasLayout>,
    #[asset(
        path = "sprites/Spritesheet/roguelikeChar_transparent.png",
        image(sampler(filter = nearest))
    )]
    pub sprites: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
struct WorldTiles {
    #[asset(texture_atlas_layout(
        tile_size_x = 16,
        tile_size_y = 16,
        padding_x = 1,
        padding_y = 1,
        rows = 31,
        columns = 57
    ))]
    pub sprite_sheet: Handle<TextureAtlasLayout>,
    #[asset(
        path = "tile-sets/Spritesheet/roguelikeSheet_transparent.png",
        image(sampler(filter = nearest))
    )]
    pub tiles: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AssetLoading {
    #[default]
    Loading,
    Loaded,
}

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

    let log_log_info = move || {
        info!("default log level is: {level}");
        info!("default log filter: \"{filter}\"");
    };

    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(ClearColor(Color::linear_rgb(0.1, 0.1, 0.1)))
        .add_plugins((
            default_plugins,
            SkeinPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            ProgressPlugin::<AssetLoading>::new()
                .with_state_transition(AssetLoading::Loading, AssetLoading::Loaded),
        ))
        .init_state::<AssetLoading>()
        .add_loading_state(
            LoadingState::new(AssetLoading::Loading)
                .continue_to_state(AssetLoading::Loaded)
                .load_collection::<WorldTiles>()
                .load_collection::<SpriteTiles>(),
        )
        // .add_plugins(FpsTrackingPlugin)
        // .add_plugins(SpherePlugin)
        .add_plugins(BasePlugin)
        .add_systems(
            OnEnter(AssetLoading::Loaded),
            || -> Progress { true.into() }.track_progress::<AssetLoading>(),
        )
        .add_systems(
            Update,
            (
                print_progress,
                track_fake_long_task.track_progress::<AssetLoading>(),
            )
                .chain()
                .run_if(in_state(AssetLoading::Loading))
                .after(LoadingStateSet(AssetLoading::Loading)),
        )
        // .add_plugins(PlayerPlugin)
        // logs log level and filters
        .add_systems(Startup, log_log_info)
        .run();
}

fn track_fake_long_task() -> Progress {
    false.into()
}

fn print_progress(
    progress: Res<ProgressTracker<AssetLoading>>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_done: Local<u32>,
) {
    let progress = progress.get_global_progress();
    if progress.done > *last_done {
        *last_done = progress.done;
        info!(
            "[Frame {}] Changed progress: {:?}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|diagnostic| diagnostic.value().unwrap_or(0.))
                .unwrap_or(0.),
            progress
        );
    }
}

// fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn(Camera2d);
//
//     let map_handle = tiled::TiledMapHandle(asset_server.load("zone-1.tmx"));
//
//     commands.spawn(tiled::TiledMapBundle {
//         tiled_map: map_handle,
//         ..Default::default()
//     });
// }
