use std::f32;

use avian3d::prelude::*;
use bevy::{
    gltf::GltfMeshExtras,
    light::CascadeShadowConfigBuilder,
    prelude::*,
    scene::{SceneInstance, SceneInstanceReady},
};
use bevy_tnua::{
    builtins::{TnuaBuiltinClimbConfig, TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use serde::{Deserialize, Serialize};

use crate::{ControlScheme, ControlSchemeConfig};

#[derive(Component)]
pub struct PlayerMeshMark;

#[derive(Component)]
pub struct TerainSceneMeshMark;

#[derive(Component)]
pub struct TerainMeshMark;

#[derive(Component, Debug)]
pub struct FloorLevel(i32);

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
        app.add_systems(Startup, setup);
        app.add_systems(Update, camera_track_player);
        app.add_systems(Update, make_higher_floors_transparent);
        // app.add_systems(Update, mark_floor_meshses);
        app.add_systems(Update, player_movement.in_set(TnuaUserControlsSystems));
        // app.add_observer(spawn_gltf_objects);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut control_scheme_configs: ResMut<Assets<ControlSchemeConfig>>,
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
    // let mesh_handle: Handle<Mesh> = asset_server.load("net-lv-01.glb#Scene0");
    // let mesh_handle: Handle<Mesh> = asset_server.load("net-lv-01.glb#Mesh0");
    let scene_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("net-lv-01.glb"));

    // terain
    // let mut entity_id =
    commands
        .spawn((
            SceneRoot(scene_handle),
            // Mesh3d(mesh_handle),
            ColliderConstructorHierarchy::new(
                // ColliderConstructor::ConvexDecompositionFromMeshWithConfig(VhacdParameters {
                //     resolution: 256,
                //     concavity: 0.00025,
                //     ..default()
                // }),
                ColliderConstructor::TrimeshFromMeshWithConfig(
                                // VhacdParameters {
                                //     resolution: 256,
                                //     concavity: 0.00025,
                                //     ..default()
                                // },
                                TrimeshFlags::empty()
                            ),

            )
            .with_default_layers(CollisionLayers::new(
                GameCollisionLayer::Terrain,
                [GameCollisionLayer::Player],
            )),
            CollisionEventsEnabled,
            RigidBody::Static,
            Friction::new(0.5).with_combine_rule(CoefficientCombine::Multiply),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Multiply),
            Transform::from_xyz(0., 0., 0.),
            TerainSceneMeshMark,
            Visibility::Hidden,
        ))
        .observe(spawn_gltf_objects)
        // .observe(mark_floor_meshses);
    ;

    // entity_id.observe(spawn_gltf_objects);

    // debug!("terain entity_id = {}", entity_id.id());

    // player
    commands
        .spawn((
            SceneRoot(asset_server.load("temp-char.glb#Scene0")),
            // Mesh3d(asset_server.load("temp-char.glb#Mesh0")),
            // ColliderConstructor::ConvexDecompositionFromMesh,
            RigidBody::Dynamic,
            TnuaController::<ControlScheme>::default(),
            Friction::new(0.425).with_combine_rule(CoefficientCombine::Multiply),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Multiply),
            // GravityScale(2.0),
            TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
                basis: TnuaBuiltinWalkConfig {
                    // The `float_height` must be greater (even if by little) from the distance between
                    // the character's center and the lowest point of its collider.
                    float_height: 1.5,
                    // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they
                    // have sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn
                    // what they do.
                    speed: 10000.,
                    acceleration: f32::INFINITY,
                    // max_slope: 2.0 * PI / 3.,
                    // max_slope: PI / 6.,
                    ..Default::default()
                },
                jump: TnuaBuiltinJumpConfig {
                    // The height is the only mandatory field of the jump action.
                    height: 4.0,
                    // `TnuaBuiltinJump` also has customization fields with sensible defaults.
                    ..Default::default()
                },
                climb: TnuaBuiltinClimbConfig { ..default() },
            })),
            // A sensor shape is not strictly necessary, but without it we'll get weird results.
            TnuaAvian3dSensorShape(Collider::cylinder(0.49, 1.0)),
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
            PlayerMeshMark,
        ))
        .with_child((
            RayCaster::new(Vec3::ZERO, Dir3::NEG_Y)
                .with_max_hits(1)
                .with_max_distance(10.),
            Transform::from_xyz(0.0, 5.0, 0.0),
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

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController<ControlScheme>>,
    // mut vel: Query<(&mut LinearVelocity, &mut AngularVelocity), With<PlayerMeshMark>>,
    // mut vel: Query<&mut LinearVelocity, With<PlayerMeshMark>>,
) {
    let Ok(mut controller) = query.single_mut() else {
        warn!("TnuaController not found, therefor not running player_movement");
        return;
    };
    controller.initiate_action_feeding();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        // info!("moving up");
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        // info!("moving down");
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        // info!("moving left");
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        // info!("moving right");
        direction += Vec3::X;
    }

    // for mut lin_velocity in vel.iter_mut() {
    //     if !keyboard.pressed(KeyCode::ArrowUp)
    //         && !keyboard.pressed(KeyCode::KeyW)
    //         && !keyboard.pressed(KeyCode::ArrowDown)
    //         && !keyboard.pressed(KeyCode::KeyS)
    //     {
    //         lin_velocity.0.z = 0.0;
    //     }
    //     if !keyboard.pressed(KeyCode::ArrowLeft)
    //         && !keyboard.pressed(KeyCode::KeyA)
    //         && !keyboard.pressed(KeyCode::ArrowRight)
    //         && !keyboard.pressed(KeyCode::KeyD)
    //     {
    //         lin_velocity.0.x = 0.0;
    //     }
    // }

    // Set the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO` to reset the previous frame's input.
    controller.basis = TnuaBuiltinWalk {
        // The `desired_motion` determines how the character will move.
        desired_motion: direction.normalize_or_zero(),
        // The other field is `desired_forward` - but since the character model is a capsule we
        // don't care the direction its "forward" is pointing.
        desired_forward: Dir3::new(direction).ok(),
        ..Default::default()
    };

    // // Feed the jump action every frame as long as the player holds the jump button. If the player
    // // stops holding the jump button, simply stop feeding the action.
    // if keyboard.pressed(KeyCode::Space) {
    //     // info!("jump");
    //     controller.action(ControlScheme::Jump(Default::default()));
    // }
}

// fn mark_floor_meshses(
//     event: On<SceneInstanceReady>,
//     mut commands: Commands,
//     // Query children of the scene or specific named entities
//     // query: Query<Entity, (With<Handle<Mesh>>, Added<Handle<StandardMaterial>>)>,
//     query: Query<&GltfMeshExtras>,
//     // query: Query<(Entity, &GltfMeshExtras)>,
//     // mut materials: ResMut<Assets<StandardMaterial>>,
//     // children: Query<&Children>,
//     // // mut meshes: ResMut<Assets<Mesh>>,
//     // mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
//     // asset_server: Res<AssetServer>,
// ) {
//     debug!("mark floor meshses");
//     let Ok(extras) = query.get(event.entity) else {
//         error!("didn't find extras");
//         return;
//     };
//     let Ok(json) = serde_json::from_str::<TerainGltfExtras>(&extras.value) else {
//         error!("invalid json in gtf extras");
//         return;
//     };
//     info!("dealing with floor level: {}", json.floor_level);
//
//     commands
//         .entity(event.entity)
//         .insert(FloorLevel(json.floor_level));
// }

fn make_higher_floors_transparent(
    // mut commands: Commands,
    // collisions: Collisions,
    // contact_graph: Res<ContactGraph>,
    // Query children of the scene or specific named entities
    // query: Query<Entity, (With<Handle<Mesh>>, Added<Handle<StandardMaterial>>)>,
    // terain: Query<
    //     (Entity, /* &FloorLevel, */ &Transform),
    //     (With<TerainSceneMeshMark>, Without<PlayerMeshMark>),
    // >,
    // terain: Query<
    //     (
    //         Entity,
    //         // &FloorLevel,
    //         // &GlobalTransform,
    //         // &ColliderTransform,
    //         &avian3d::physics_transform::Position,
    //         &MeshMaterial3d<StandardMaterial>,
    //         // &GltfMaterialName,
    //     ),
    //     (
    //         With<Collider>,
    //         // With<TerainSceneMeshMark>,
    //         With<GltfMeshExtras>,
    //         // Without<PlayerMeshMark>,
    //     ),
    // >,
    // player_loc: Single<&Transform, (Without<TerainSceneMeshMark>, With<PlayerMeshMark>)>,
    // query: Query<(Entity, &GltfMeshExtras)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // children: Query<&Children>,
    // // mut meshes: ResMut<Assets<Mesh>>,
    // mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
    // asset_server: Res<AssetServer>,
    terain: Query<
        (&MeshMaterial3d<StandardMaterial>, &FloorLevel),
        (
            // With<Collider>,
            With<TerainMeshMark>,
            // Without<TerainSceneMeshMark>,
        ),
    >,
    // player_loc: Single<Entity, (Without<TerainSceneMeshMark>, With<PlayerMeshMark>)>,
    rays: Query<&RayHits>,
) {
    // for (entity, terain_tranform, material) in terain {
    //     commands.entity(entity).log_components();
    //     // debug!("{entity} :  transform => {}", terain_tranform.translation());
    //     debug!("{entity} :  transform => {}", terain_tranform.0.y);
    //
    //     let Some(material) = materials.get_mut(material.0.id()) else {
    //         error!("failed to aquire material asset");
    //         continue;
    //     };
    //     // debug!(
    //     //     "graound-loc: {} => player_loc: {}",
    //     //     terain_tranform.translation(),
    //     //     player_loc.translation
    //     // );
    //
    //     if terain_tranform.0.y > player_loc.translation.y {
    //         // debug!("making transparent");
    //         material.alpha_mode = AlphaMode::Blend;
    //         material.base_color.set_alpha(0.125);
    //     } else {
    //         // debug!("making opaque");
    //         material.base_color.set_alpha(1.0);
    //     }
    // }

    for (material, floor_level) in terain {
        let Some(material) = materials.get_mut(material.0.id()) else {
            error!("failed to aquire material asset");
            continue;
        };
        // material.base_color.set_alpha(1.0);
        material.alpha_mode = AlphaMode::Blend;
        material.base_color.set_alpha(0.125);
        // info!("floor_level => {floor_level:?}");
    }

    // for collider in contact_graph.entities_colliding_with(*player_loc) {
    //     debug!("contact_pair = {collider:?}");
    //
    //     // for collider in [contact_pair.collider1, contact_pair.collider2] {
    //     let Ok(material_id) = terain.get(collider) else {
    //         error!("failed to find material");
    //         continue;
    //     };
    //
    //     let Some(material) = materials.get_mut(material_id.0.id()) else {
    //         error!("failed to aquire material asset");
    //         continue;
    //     };
    //     // material.base_color.set_alpha(1.0);
    //     // material.alpha_mode = AlphaMode::Blend;
    //     // material.base_color.set_alpha(0.125);
    //     info!("floor_level => {:?}", material_id.1);
    //     material.base_color.set_alpha(1.0);
    //     // }
    // }

    for hits in rays {
        // if hits.iter().len() > 0 {
        //     debug!("{:?}", hits);
        // }
        for hit in hits.iter() {
            let Ok(material_id) = terain.get(hit.entity) else {
                error!("failed to find material");
                continue;
            };

            let Some(material) = materials.get_mut(material_id.0.id()) else {
                error!("failed to aquire material asset");
                continue;
            };
            info!("floor_level => {:?}", material_id.1);
            material.base_color.set_alpha(1.0);
        }
    }
}

// fn make_higher_floors_transparent(
//     // mut commands: Commands,
//     // Query children of the scene or specific named entities
//     // query: Query<Entity, (With<Handle<Mesh>>, Added<Handle<StandardMaterial>>)>,
//     // terain: Query<
//     //     (Entity, /* &FloorLevel, */ &Transform),
//     //     (With<TerainSceneMeshMark>, Without<PlayerMeshMark>),
//     // >,
//     rays: Query<
//         // Entity,
//         // // &FloorLevel,
//         // // &GlobalTransform,
//         // // &ColliderTransform,
//         // &avian3d::physics_transform::Position,
//         // &MeshMaterial3d<StandardMaterial>,
//         // // &GltfMaterialName,
//         // &RayCaster,
//         &RayHits,
//         // (
//         //     // With<Collider>,
//         //     // With<TerainSceneMeshMark>,
//         //     // With<GltfMeshExtras>,
//         // With<PlayerMeshMark>,
//         // ),
//     >,
//     // gltf_extras: Query<&GltfMeshExtras, (With<TerainMeshMark>, Without<PlayerMeshMark>)>,
//     mut terain: Query<
//         &mut MeshMaterial3d<StandardMaterial>,
//         (With<Collider>, With<TerainMeshMark>),
//     >,
//     // player_loc: Single<&Transform, (Without<TerainSceneMeshMark>, With<PlayerMeshMark>)>,
//     // query: Query<(Entity, &GltfMeshExtras)>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     // children: Query<&Children>,
//     // // mut meshes: ResMut<Assets<Mesh>>,
//     // mesh_materials: Query<(&MeshMaterial3d<StandardMaterial>, &GltfMaterialName)>,
//     // asset_server: Res<AssetServer>,
// ) {
//     // trace!("make_higher_floors_transparent");
//
//     for hits in rays {
//         // if hits.iter().len() > 0 {
//         //     debug!("{:?}", hits);
//         // }
//         for hit in hits.iter() {
//             // commands.entity(hit.entity).log_components();
//             // debug!("{entity} :  transform => {}", terain_tranform.translation());
//             // debug!("{entity} :  transform => {}", terain_tranform.0.y);
//             // debug!("{}", hit.entity);
//
//             let Ok(material_id) = terain.get_mut(hit.entity) else {
//                 error!("failed to find material");
//                 continue;
//             };
//
//             let Some(material) = materials.get_mut(material_id.0.id()) else {
//                 error!("failed to aquire material asset");
//                 continue;
//             };
//             // debug!(
//             //     "graound-loc: {} => player_loc: {}",
//             //     terain_tranform.translation(),
//             //     player_loc.translation
//             // );
//             // let mut new_material = material.clone();
//
//             // if terain_tranform.0.y > player_loc.translation.y {
//             // debug!("making transparent");
//
//             // warn!("making material id {}, translucent", material_id.0.id());
//             material.alpha_mode = AlphaMode::Blend;
//             material.base_color.set_alpha(0.125);
//             // material_id.0 = materials.add(new_material);
//
//             // } else {
//             // debug!("making opaque");
//             // material.base_color.set_alpha(1.0);
//             // }
//             // gltf_extras
//             //     .get(hit.entity)
//             //     .map(|extra_str| info!("{}", extra_str.value));
//         }
//     }
// }

fn spawn_gltf_objects(
    event: On<SceneInstanceReady>,
    mut commands: Commands,
    // mut scene_instance_ready_events: MessageReader<SceneInstanceReady>,
    // gltf_handles: Res<GltfHandle>,
    // gltf_assets: Res<Assets<Gltf>>,
    // mesh_assets: Res<Assets<Mesh>>,
    parent_query: Query<&Children>,
    mesh_material_query: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &Transform,
        ),
        // With<TerainSceneMeshMark>,
    >,
    // mesh_query: Query<&Handle<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    extras: Query<&GltfMeshExtras>,
) {
    // for event in scene_instance_ready_events.read() {
    // bevy spawn each mesh in a gltf scene independently
    // Find the root of the gltf scene
    // if let Ok(children) = children_query.get(event.entity) {
    //     for child in children.iter() {
    // Traversal might be needed if meshes are nested.
    // Simple version: child is the mesh entity
    // if let Ok((mesh, material, transform)) = mesh_material_query.get(child.entity()) {
    let root_entity = event.entity;
    warn!("event entity id = {}", root_entity);
    warn!("{} meshes found", mesh_material_query.iter().len());
    // commands.entity(root_entity).log_components();

    // for children in parent_query.iter() {
    for child in parent_query.iter_descendants(event.entity) {
        for child_entity in parent_query.iter_descendants(child) {
            if let Ok((_entity, mesh, material, transform)) = mesh_material_query.get(child_entity)
            {
                let Ok(extra_info) = extras.get(child_entity) else {
                    error!("gltf extra data not found");
                    continue;
                };

                // info!("spawning meshes: {}", child_entity);
                let material = materials.get(material).clone().unwrap().clone();
                let material = materials.add(material);
                // info!("material_id {}", material.id());

                if let Ok(json) = serde_json::from_str::<TerainGltfExtras>(&extra_info.value) {
                    info!("dealing with floor level: {}", json.floor_level);

                    commands.entity(child_entity).with_child((
                        // commands.spawn((
                        mesh.clone(),
                        MeshMaterial3d(material),
                        transform.clone(),
                        // Transform::from_xyz(0., 0., 0.),
                        ColliderConstructorHierarchy::new(
                            ColliderConstructor::TrimeshFromMeshWithConfig(
                                // VhacdParameters {
                                //     resolution: 256,
                                //     concavity: 0.00025,
                                //     ..default()
                                // },
                                TrimeshFlags::all(),
                            ),
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
                // .observe(mark_floor_meshses);
                // commands.entity(child_entity).despawn();
            } else {
                // commands.entity(child_entity).log_components();
                // error!(
                //     "child {} did not have the proper components, {:?}",
                //     child_entity,
                //     mesh_material_query.iter().map(|q| q.0).collect::<Vec<_>>()
                // );
            }
        }
    }

    // for (mesh, material, transform) in mesh_material_query {
    //     info!("spawning independed mesh");
    //     commands.spawn((
    //         mesh.clone(),
    //         material.clone(),
    //         ColliderConstructorHierarchy::new(
    //             ColliderConstructor::ConvexDecompositionFromMeshWithConfig(VhacdParameters {
    //                 resolution: 256,
    //                 concavity: 0.00025,
    //                 ..default()
    //             }),
    //         )
    //         .with_default_layers(CollisionLayers::new(
    //             GameCollisionLayer::Terrain,
    //             [GameCollisionLayer::Player],
    //         )),
    //         RigidBody::Static,
    //         *transform,
    //         TerainMeshMark,
    //         Name::new("IndependentMesh"),
    //     ));
    // }

    // else {
    //     error!(
    //         "failed to query for material from entity {}",
    //         child.entity()
    //     )
    // }
    //     }
    //     // Optional: despawn the original scene root
    //     // commands.entity(event.parent).despawn_recursive();
    // }
    // }
}
