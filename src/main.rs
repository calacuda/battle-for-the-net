use bevy::{
    diagnostic::{
        // DiagnosticsStore,
        FrameTimeDiagnosticsPlugin,
    },
    light::DirectionalLightShadowMap,
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_ecs_tiled::prelude::*;
use bevy_skein::SkeinPlugin;

use crate::helper::DisplayMapPlugin;

// use bevy_ecs_tilemap::prelude::*;
// use bevy_ecs_tiled::debug::*;
// use iyes_progress::{Progress, ProgressTracker};
// use bevy_spritefusion::prelude::*;
// use bevy_asset_loader::prelude::*;

// pub mod base;
pub mod helper;

// #[derive(AssetCollection, Resource)]
// struct SpriteTiles {
//     // #[asset(path = "../assets/tile-sets/single-png/", collection(mapped, typed), image(sampler(filter = nearest)))]
//     // floor: HashMap<AssetFileStem, Handle<Image>>,
//     // #[asset(path = "../assets/sprites/single-png/", collection(mapped, typed), image(sampler(filter = nearest)))]
//     // sprites: HashMap<AssetFileStem, Handle<Image>>,
//     #[asset(texture_atlas_layout(
//         tile_size_x = 16,
//         tile_size_y = 16,
//         padding_x = 1,
//         padding_y = 1,
//         rows = 12,
//         columns = 54
//     ))]
//     pub sprite_sheet: Handle<TextureAtlasLayout>,
//     #[asset(
//         path = "sprites/Spritesheet/roguelikeChar_transparent.png",
//         image(sampler(filter = nearest))
//     )]
//     pub sprites: Handle<Image>,
// }
//
// #[derive(AssetCollection, Resource)]
// struct WorldTiles {
//     #[asset(texture_atlas_layout(
//         tile_size_x = 16,
//         tile_size_y = 16,
//         padding_x = 1,
//         padding_y = 1,
//         rows = 31,
//         columns = 57
//     ))]
//     pub sprite_sheet: Handle<TextureAtlasLayout>,
//     #[asset(
//         path = "tile-sets/Spritesheet/roguelikeSheet_transparent.png",
//         image(sampler(filter = nearest))
//     )]
//     pub tiles: Handle<Image>,
// }

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

    let default_plugins = DefaultPlugins
        .set(LogPlugin {
            // Set the default log level for everything
            level,
            // and use a filter string for fine-grained control
            filter: filter.clone(),
            ..default()
        })
        .set(ImagePlugin::default_nearest());

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
            // ProgressPlugin::<AssetLoading>::new()
            //     .with_state_transition(AssetLoading::Loading, AssetLoading::Loaded),
        ))
        .add_plugins(TiledPlugin::default())
        .add_plugins(DisplayMapPlugin)
        // .add_plugins(TiledDebugPluginGroup)
        // .add_plugins(BasePlugin)
        .add_systems(Startup, (startup,))
        .add_systems(Update, switch_map)
        // .init_state::<AssetLoading>()
        // .add_loading_state(
        //     LoadingState::new(AssetLoading::Loading).continue_to_state(AssetLoading::Loaded), // .load_collection::<WorldTiles>()
        //                                                                                       // .load_collection::<SpriteTiles>(),
        // )
        // .add_systems(
        //     OnEnter(AssetLoading::Loaded),
        //     || -> Progress { true.into() }.track_progress::<AssetLoading>(),
        // )
        // .add_systems(
        //     Update,
        //     (
        //         print_progress,
        //         track_fake_long_task.track_progress::<AssetLoading>(),
        //     )
        //         .chain()
        //         .run_if(in_state(AssetLoading::Loading))
        //         .after(LoadingStateSet(AssetLoading::Loading)),
        // )
        // logs log level and filters
        .add_systems(Startup, log_log_info)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let default_callback: helper::assets::MapInfosCallback = |c| {
        info!("default_callback");
        c.insert((
            TilemapAnchor::Center,
            // For isometric maps, it can be useful to tweak `bevy_ecs_tilemap` render settings.
            // [`TilemapRenderSettings`] provides the `y_sort`` parameter to sort chunks using their y-axis
            // position during rendering.
            // However, it applies to whole chunks, not individual tile, so we have to force the chunk
            // size to be exactly one tile along the y-axis.
            TilemapRenderSettings {
                render_chunk_size: UVec2::new(23, 1),
                y_sort: true,
            },
        ));
    };

    // The `helper::AssetsManager` struct is an helper to easily switch between maps in examples.
    // You should NOT use it directly in your games.
    let mut mgr = helper::assets::AssetsManager::new(&mut commands);
    mgr.add_map(helper::assets::MapInfos::new(
        &asset_server,
        "zone-1.3.tmx",
        // "A finite 'diamond' isometric map",
        "version 3 of the map. map is 'diamond' isometric map",
        default_callback,
    ));
    // mgr.add_map(helper::assets::MapInfos::new(
    //     &asset_server,
    //     "zone-1.2.tmx",
    //     "version 2 of the map. map is 'diamond' isometric map",
    //     default_callback,
    // ));
    mgr.cycle_map(&mut commands);

    commands.insert_resource(mgr);
}

fn switch_map(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mgr: ResMut<helper::assets::AssetsManager>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        mgr.cycle_map(&mut commands);
    }
}

// fn track_fake_long_task() -> Progress {
//     false.into()
// }
//
// fn print_progress(
//     progress: Res<ProgressTracker<AssetLoading>>,
//     diagnostics: Res<DiagnosticsStore>,
//     mut last_done: Local<u32>,
// ) {
//     let progress = progress.get_global_progress();
//     if progress.done > *last_done {
//         *last_done = progress.done;
//         info!(
//             "[Frame {}] Changed progress: {:?}",
//             diagnostics
//                 .get(&FrameTimeDiagnosticsPlugin::FRAME_COUNT)
//                 .map(|diagnostic| diagnostic.value().unwrap_or(0.))
//                 .unwrap_or(0.),
//             progress
//         );
//     }
// }
