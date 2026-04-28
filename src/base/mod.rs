use std::f32::{
    self,
    consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8, PI},
};

use avian3d::prelude::*;
use bevy::{
    camera::primitives::Aabb, gltf::GltfMeshExtras, light::CascadeShadowConfigBuilder, prelude::*,
    scene::SceneInstanceReady,
};
// use bevy_tnua::{
//     TnuaObstacleRadar,
//     builtins::{
//         TnuaBuiltinClimb, TnuaBuiltinClimbConfig, TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig,
//     },
//     prelude::*,
//     radar_lens::{TnuaBlipSpatialRelation, TnuaRadarLens},
// };
// use bevy_tnua_avian3d::{TnuaAvian3dSensorShape, TnuaSpatialExtAvian3d};
use serde::{Deserialize, Serialize};

// use crate::{ControlScheme, ControlSchemeConfig};

#[derive(Component)]
pub struct PlayerMeshMark;

#[derive(Component)]
pub struct TerainSceneMeshMark;

#[derive(Component)]
pub struct TerainMeshMark;

#[derive(Component, Default, Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub struct FloorLevel(pub i32);

#[derive(Resource, Default, Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub struct PlayerOnFloor(pub FloorLevel);

#[derive(Component, Deserialize, Serialize)]
pub struct TerainGltfExtras {
    #[serde(rename = "FloorLevel")]
    floor_level: i32,
}

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default,
    Player,
    Terrain,
}

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        trace!("added base plugin.");
        app.init_resource::<PlayerOnFloor>();
        app.add_systems(Startup, setup);
        // app.add_systems(Update, camera_track_player);
        // app.add_systems(Update, make_higher_floors_transparent);
        app.add_systems(
            Update,
            (
                // player_movement, // .in_set(TnuaUserControlsSystems),
                (
                    make_higher_floors_transparent, // .in_set(TnuaUserControlsSystems),
                    camera_track_player,
                ),
            )
                .chain(),
        );
        app.add_systems(Update, print_aabb_height_system);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut control_scheme_configs: ResMut<Assets<ControlSchemeConfig>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(21., 21., 42.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
        Transform::from_xyz(10.5, 10.5, 42.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));

    let transform = Transform::from_xyz(0.0, 10.0, 0.0);
    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("net-lv-01.glb"));

    let friction = Friction::new(0.125).with_combine_rule(CoefficientCombine::Average);

    // terain
    commands
        .spawn((
            SceneRoot(scene_handle),
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMeshWithConfig(
                TrimeshFlags::empty(),
            ))
            .with_default_layers(CollisionLayers::new(
                GameCollisionLayer::Terrain,
                [GameCollisionLayer::Player],
            )),
            CollisionEventsEnabled,
            RigidBody::Static,
            // Friction::new(0.5).with_combine_rule(CoefficientCombine::Multiply),
            friction,
            // Restitution::new(-1.0).with_combine_rule(CoefficientCombine::Multiply),
            Restitution::new(-1.0).with_combine_rule(CoefficientCombine::Average),
            Transform::from_xyz(0., 0., 0.),
            TerainSceneMeshMark,
            Visibility::Hidden,
        ))
        .observe(spawn_gltf_objects);

    // player
    commands.spawn((
        SceneRoot(asset_server.load("temp-char.glb#Scene0")),
        // ColliderConstructor::ConvexDecompositionFromMesh,
        RigidBody::Dynamic,
        // TnuaController::<ControlScheme>::default(),
        // Friction::new(0.125).with_combine_rule(CoefficientCombine::Average),
        // Friction::new(0.45).with_combine_rule(CoefficientCombine::Multiply),
        friction,
        Restitution::new(-1.0).with_combine_rule(CoefficientCombine::Average),
        // GravityScale(2.0),
        // GravityScale(0.03125),
        // TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
        //     basis: TnuaBuiltinWalkConfig {
        //         // The `float_height` must be greater (even if by little) from the distance between
        //         // the character's center and the lowest point of its collider.
        //         float_height: 1.0,
        //         // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they
        //         // have sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn
        //         // what they do.
        //         speed: 10000.,
        //         acceleration: f32::INFINITY,
        //         // // max_slope: 2.0 * PI / 3.,
        //         // // max_slope: PI / 6.,
        //         // max_slope: std::f32::consts::FRAC_PI_2,
        //         // max_slope: 5.0 * PI / 6.,
        //         // max_slope: PI,
        //         // max_slope: (PI) / 2.,
        //         cling_distance: 100.0,
        //         spring_strength: 100.,
        //         ..Default::default()
        //     },
        //     jump: TnuaBuiltinJumpConfig {
        //         // The height is the only mandatory field of the jump action.
        //         height: 4.0,
        //         // `TnuaBuiltinJump` also has customization fields with sensible defaults.
        //         ..Default::default()
        //     },
        //     climb: TnuaBuiltinClimbConfig {
        //         climb_speed: 10.,
        //         climb_acceleration: f32::INFINITY,
        //         ..default()
        //     },
        // })),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        // TnuaAvian3dSensorShape(Collider::cylinder(0.49, 1.0)),
        // TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED,
        transform,
        CollisionEventsEnabled,
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
            .with_default_layers(CollisionLayers::new(
                GameCollisionLayer::Player,
                [GameCollisionLayer::Terrain],
            )),
        // TnuaObstacleRadar::new(0.6, 1.0),
        // TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        RayCaster::new(Vec3::ZERO, Dir3::NEG_Y)
            .with_max_hits(1)
            .with_max_distance(15.),
        PlayerMeshMark,
    ));
}

pub fn camera_track_player(
    // time: Res<Time>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<PlayerMeshMark>)>,
    player_transform: Single<Option<&Transform>, With<PlayerMeshMark>>,
) {
    let Some(player_transform) = player_transform.into_inner() else {
        warn!("player not found");
        return;
    };

    for mut cam_tranform in camera.iter_mut() {
        cam_tranform.translation = player_transform.translation;
        cam_tranform.translation += Vec3::new(21., 21., 42.);
        cam_tranform.look_at(player_transform.translation, Vec3::Y);
    }
}

// fn player_movement(
//     keyboard: Res<ButtonInput<KeyCode>>,
//     mut query: Query<&mut TnuaController<ControlScheme>, With<PlayerMeshMark>>,
//     // mut query: Query<(&mut TnuaController<ControlScheme>, &TnuaObstacleRadar)>,
//     // mut vel: Query<(&mut LinearVelocity, &mut AngularVelocity), With<PlayerMeshMark>>,
//     // mut vel: Query<&mut LinearVelocity, With<PlayerMeshMark>>,
//     // spatial_ext: TnuaSpatialExtAvian3d,
//     // player_transform: Single<Option<&Transform>, With<PlayerMeshMark>>,
// ) {
//     let Ok(mut controller) = query.single_mut() else {
//         // let Ok((mut controller, obstacle_radar)) = query.single_mut() else {
//         warn!("TnuaController not found, not running player_movement");
//         return;
//     };
//     controller.initiate_action_feeding();
//
//     let mut direction = Vec3::ZERO;
//     // direction.y += FRAC_PI_6;
//
//     if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
//         // info!("moving up");
//         direction -= Vec3::Z;
//     }
//     if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
//         // info!("moving down");
//         direction += Vec3::Z;
//     }
//     if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
//         // info!("moving left");
//         direction -= Vec3::X;
//     }
//     if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
//         // info!("moving right");
//         direction += Vec3::X;
//     }
//
//     // for mut lin_velocity in vel.iter_mut() {
//     //     if !keyboard.pressed(KeyCode::ArrowUp)
//     //         && !keyboard.pressed(KeyCode::KeyW)
//     //         && !keyboard.pressed(KeyCode::ArrowDown)
//     //         && !keyboard.pressed(KeyCode::KeyS)
//     //     {
//     //         lin_velocity.0.z = 0.0;
//     //     }
//     //     if !keyboard.pressed(KeyCode::ArrowLeft)
//     //         && !keyboard.pressed(KeyCode::KeyA)
//     //         && !keyboard.pressed(KeyCode::ArrowRight)
//     //         && !keyboard.pressed(KeyCode::KeyD)
//     //     {
//     //         lin_velocity.0.x = 0.0;
//     //     }
//     // }
//
//     // Set the basis every frame. Even if the player doesn't move - just use `desired_velocity:
//     // Vec3::ZERO` to reset the previous frame's input.
//     controller.basis = TnuaBuiltinWalk {
//         // The `desired_motion` determines how the character will move.
//         desired_motion: direction,
//         // The other field is `desired_forward` - but since the character model is a capsule we
//         // don't care the direction its "forward" is pointing.
//         desired_forward: Dir3::new(direction).ok(),
//         // ..Default::default()
//     };
//
//     // let radar_lens = TnuaRadarLens::new(obstacle_radar, &spatial_ext);
//     //
//     // for blip in radar_lens.iter_blips() {
//     //     if let TnuaBlipSpatialRelation::Aeside(blip_direction) = blip.spatial_relation(0.25) {
//     //         let dot = blip_direction.dot(direction);
//     //         let blip_direction = blip_direction.to_owned().as_vec3().to_owned();
//     //         let should_climb =
//     //         // (-0.75 > blip_direction.dot(direction)
//     //         //     || 0.75 < blip_direction.dot(direction))
//     //             // && -1.0 != blip_direction.dot(direction)
//     //             // && 1.0 != blip_direction.dot(direction)
//     //             // && blip_direction.y <= 0.01
//     //             // && blip_direction.y >= -0.01
//     //             // &&
//     //             dot != 0.0 &&
//     //             direction != Vec3::ZERO &&
//     //             ((0.75 <= blip_direction.x.abs()
//     //                 && blip_direction.z == 0.0)
//     //                 || (0.75 <= blip_direction.z.abs()
//     //                     && blip_direction.x == 0.0));
//     //
//     //         // let should_climb = blip_direction.x >;
//     //
//     //         if should_climb {
//     //             info!("Climb");
//     //             // warn!(
//     //             //     "dot: {} | blip_dir: {}",
//     //             //     blip_direction.dot(direction),
//     //             //     blip_direction.as_vec3()
//     //             // );
//     //             let desired_climb_motion = direction; // + Vec3::Y;
//     //             // desired_climb_motion.y += FRAC_PI_3;
//     //             // desired_climb_motion.y = FRAC_PI_3;
//     //             info!("climb_motion: {desired_climb_motion}");
//     //             let player_loc = player_transform.unwrap().translation;
//     //
//     //             controller.action(ControlScheme::Climb(TnuaBuiltinClimb {
//     //                 anchor: player_loc,
//     //                 desired_vec_to_anchor: direction + player_loc,
//     //                 desired_climb_motion,
//     //                 // desired_forward: Dir3::new(blip_direction).ok(),
//     //                 // hard_stop_up: (),
//     //                 // hard_stop_down: (),
//     //                 ..Default::default()
//     //             }));
//     //         }
//     //
//     //         warn!(
//     //             "dot: {} | blip_dir: {}",
//     //             blip_direction.dot(direction),
//     //             blip_direction
//     //         );
//     //     }
//     // }
//
//     // Feed the jump action every frame as long as the player holds the jump button. If the player
//     // stops holding the jump button, simply stop feeding the action.
//     if keyboard.pressed(KeyCode::Space) {
//         // info!("jump");
//         controller.action(ControlScheme::Jump(Default::default()));
//     }
// }

fn make_higher_floors_transparent(
    mut materials: ResMut<Assets<StandardMaterial>>,
    terain: Query<(&MeshMaterial3d<StandardMaterial>, &FloorLevel), (With<TerainMeshMark>,)>,
    rays: Query<&RayHits>,
    mut floor: ResMut<PlayerOnFloor>,
) {
    let mut mk_solid: Vec<_> = Vec::new();

    for hits in rays {
        for hit in hits.iter() {
            let Ok((_material_id, level)) = terain.get(hit.entity) else {
                error!("failed to find material");
                continue;
            };
            mk_solid.push(level);
            floor.0 = *level;
            break;
        }
    }

    let mk_solid = mk_solid.first();

    for (material, floor_level) in terain {
        if mk_solid.is_some_and(|on_level| {
            if (on_level.0 % 2) == 0 {
                (on_level.0 + 1 >= floor_level.0) && (on_level.0 - 1 <= floor_level.0)
            } else {
                (on_level.0 == floor_level.0) || (on_level.0 - 1 == floor_level.0)
            }
        }) {
            let Some(material) = materials.get_mut(material.0.id()) else {
                error!("failed to aquire material asset");
                continue;
            };
            material.alpha_mode = AlphaMode::Opaque;
            material.base_color.set_alpha(1.0);
            // debug!("floor_level => {floor_level:?}");
        } else {
            let Some(material) = materials.get_mut(material.0.id()) else {
                error!("failed to aquire material asset");
                continue;
            };
            material.alpha_mode = AlphaMode::Blend;
            material.base_color.set_alpha(0.1);
        }
    }
}

fn spawn_gltf_objects(
    event: On<SceneInstanceReady>,
    mut commands: Commands,
    parent_query: Query<&Children>,
    mesh_material_query: Query<(
        Entity,
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &Transform,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    extras: Query<&GltfMeshExtras>,
) {
    // Traversal might be needed if meshes are nested.
    // Simple version: child is the mesh entity
    let root_entity = event.entity;
    warn!("event entity id = {}", root_entity);
    warn!("{} meshes found", mesh_material_query.iter().len());

    // for children in parent_query.iter() {
    for child in parent_query.iter_descendants(event.entity) {
        for child_entity in parent_query.iter_descendants(child) {
            if let Ok((_entity, mesh, material, transform)) = mesh_material_query.get(child_entity)
            {
                let Ok(extra_info) = extras.get(child_entity) else {
                    error!("gltf extra data not found");
                    continue;
                };

                let material = materials.get(material).unwrap().clone();
                let material = materials.add(material);

                if let Ok(json) = serde_json::from_str::<TerainGltfExtras>(&extra_info.value) {
                    info!("dealing with floor level: {}", json.floor_level);

                    commands.entity(child_entity).with_child((
                        mesh.clone(),
                        MeshMaterial3d(material),
                        *transform,
                        ColliderConstructorHierarchy::new(
                            ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::all()),
                        )
                        .with_default_layers(CollisionLayers::new(
                            GameCollisionLayer::Terrain,
                            [GameCollisionLayer::Player],
                        )),
                        CollisionEventsEnabled,
                        Friction::new(0.5).with_combine_rule(CoefficientCombine::Multiply),
                        Restitution::ZERO.with_combine_rule(CoefficientCombine::Multiply),
                        RigidBody::Static,
                        TerainMeshMark,
                        Visibility::Visible,
                        FloorLevel(json.floor_level),
                    ));
                } else {
                    error!("invalid json in gtf extras {}", extra_info.value);
                    continue;
                };
            }
        }
    }
}

fn print_aabb_height_system(query: Query<&Aabb>) {
    for aabb in query.iter() {
        // Aabb.half_extents is the distance from center to edge
        let height = aabb.half_extents.y * 2.0;
        println!("Mesh AABB height: {}", height);
    }
}
