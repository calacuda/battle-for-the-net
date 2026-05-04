use avian3d::prelude::*;
use bevy::{
    gltf::GltfMeshExtras, light::CascadeShadowConfigBuilder, prelude::*, scene::SceneInstanceReady,
};
use bevy_ahoy::prelude::*;
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

use crate::PlayerInput;

#[derive(Component)]
pub struct PlayerMeshMark;

#[derive(Component)]
pub struct TerainSceneMeshMark;

#[derive(Component)]
pub struct TerainMeshMark;

#[derive(Component, Deref, DerefMut, Default, Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub struct FloorLevel(pub i32);

#[derive(Resource, Deref, DerefMut, Default, Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
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
        app.add_input_context::<PlayerInput>();
        app.add_systems(
            Update,
            (make_higher_floors_transparent, camera_track_player),
        );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    let friction = Friction::new(10.0).with_combine_rule(CoefficientCombine::Average);

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
        PlayerInput,
        CharacterController {
            speed: 7.75,
            ..CharacterController::default()
        },
        actions!(PlayerInput[
            (
                Action::<Movement>::new(),
                // Normalize the input vector
                DeadZone::default(),
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Axial::left_stick()
                ))
            ),
            (
                Action::<Jump>::new(),
                bindings![KeyCode::Space,  GamepadButton::South],
            ),
            (
                Action::<Crouch>::new(),
                bindings![KeyCode::ControlLeft, GamepadButton::LeftTrigger2],
            ),
            (
                Action::<RotateCamera>::new(),
                Bindings::spawn((
                    // tweak mouse and right stick sensitivity
                    // in Scale::splat values
                    Spawn((Binding::mouse_motion(), Scale::splat(0.07))),
                    Axial::right_stick().with((Scale::splat(4.0), DeadZone::default())),
                ))
            ),
        ]),
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

            if floor.0 != *level {
                info!("on floor {}", level.0);
            }

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
