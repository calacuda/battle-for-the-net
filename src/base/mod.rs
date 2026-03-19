// use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use bevy::prelude::*;

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        info!("added base plugin.");
        // app.add_systems(Startup, setup);
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let map_handle = crate::tiled::TiledMapHandle(asset_server.load("zone-1.tmx"));

    commands.spawn(crate::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

// fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//     let transform =
//         Transform::from_xyz(2.0, 2.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y);
//
//     commands.spawn((
//         Camera3d::default(),
//         transform,
//         // EnvironmentMapLight {
//         //     diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
//         //     specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
//         //     intensity: 250.0,
//         //     ..default()
//         // },
//     ));
//     commands.spawn((
//         DirectionalLight {
//             shadows_enabled: true,
//             ..default()
//         },
//         transform,
//         // This is a relatively small scene, so use tighter shadow
//         // cascade bounds than the default for better quality.
//         // We also adjusted the shadow map to be larger since we're
//         // only using a single cascade.
//         CascadeShadowConfigBuilder {
//             num_cascades: 1,
//             maximum_distance: 1.6,
//             ..default()
//         }
//         .build(),
//     ));
//
//     commands.spawn((
//         SceneRoot(
//             asset_server.load(
//                 // GltfAssetLabel::Scene(0)
//                 // .from_asset("3d-models/kenney_animated-characters-2/Model/characterMedium.fbx"),
//                 "3d-models/kenney_3d-road-tiles/Models/gLTF/roadTile_042.gltf#Scene0",
//             ),
//             // ),
//         ),
//         // Material(
//         //     asset_server
//         //         .load("3d-models/kenney_3d-road-tiles/Models/gLTF/roadTile_001.gltf#material0"),
//         // ),
//         Transform::from_xyz(0.0, 0.0, 0.0),
//     ));
// }
