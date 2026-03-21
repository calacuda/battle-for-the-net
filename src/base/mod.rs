use avian3d::{math::PI, prelude::*};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use bevy_tnua::{
    builtins::{TnuaBuiltinJumpConfig, TnuaBuiltinWalkConfig},
    prelude::*,
};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
// use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
// use bevy_tnua_avian3d::prelude::*;

use crate::{ControlScheme, ControlSchemeConfig};

#[derive(Component)]
pub struct PlayerMeshMark;

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
        info!("added base plugin.");
        app.add_systems(Startup, setup);
        app.add_systems(Update, camera_track_player);
        app.add_systems(Update, player_movement.in_set(TnuaUserControlsSystems));
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

    let transform = Transform::from_xyz(0.0, 1.5, 0.0);
    // let mesh_handle: Handle<Mesh> = asset_server.load("net-lv-01.glb#Scene0");
    let mesh_handle: Handle<Mesh> = asset_server.load("net-lv-01.glb#Mesh0");
    let scene_handle: Handle<Scene> = asset_server.load("net-lv-01.glb#Scene0");

    // terain
    commands.spawn((
        SceneRoot(scene_handle),
        Mesh3d(mesh_handle),
        // ColliderConstructor::ConvexDecompositionFromMesh,
        // Collider::half_space(Vec3::Y * 2.0),
        ColliderConstructorHierarchy::new(
            ColliderConstructor::ConvexDecompositionFromMeshWithConfig(VhacdParameters {
                resolution: 256,
                concavity: 0.00025,
                ..default()
            }),
        )
        .with_default_layers(CollisionLayers::new(
            GameCollisionLayer::Terrain,
            [GameCollisionLayer::Player],
        )),
        RigidBody::Static,
        // CollisionLayers::new(GameCollisionLayer::Terrain, GameCollisionLayer::Player),
        // transform,
    ));
    // player
    commands.spawn((
        SceneRoot(asset_server.load("temp-char.glb#Scene0")),
        Mesh3d(asset_server.load("temp-char.glb#Mesh0")),
        // ColliderConstructor::ConvexDecompositionFromMesh,
        RigidBody::Dynamic,
        TnuaController::<ControlScheme>::default(),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Multiply),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Multiply),
        // GravityScale(2.0),
        TnuaConfig::<ControlScheme>(control_scheme_configs.add(ControlSchemeConfig {
            basis: TnuaBuiltinWalkConfig {
                // The `float_height` must be greater (even if by little) from the distance between
                // the character's center and the lowest point of its collider.
                float_height: 2.5,
                // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they
                // have sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn
                // what they do.
                // speed: 50.,
                // max_slope: 2.0 * PI / 3.,
                // acceleration: 1.,
                ..Default::default()
            },
            // jump: TnuaBuiltinJumpConfig {
            //     // The height is the only mandatory field of the jump action.
            //     height: 4.0,
            //     // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            //     ..Default::default()
            // },
        })),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 1.0)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED,
        transform,
        ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
            .with_default_layers(CollisionLayers::new(
                GameCollisionLayer::Player,
                [GameCollisionLayer::Terrain],
            )),
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

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController<ControlScheme>>,
    mut vel: Query<(&mut LinearVelocity, &mut AngularVelocity), With<PlayerMeshMark>>,
) {
    let Ok(mut controller) = query.single_mut() else {
        warn!("TnuaController not found, therefor not running player_movement");
        return;
    };
    controller.initiate_action_feeding();

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        // info!("moving up");
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        // info!("moving down");
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        // info!("moving left");
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        // info!("moving right");
        direction += Vec3::X;
    }

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

    if direction == Vec3::ZERO {
        // controller.basis_config
        for (mut lin_velocity, mut ang_velocity) in vel.iter_mut() {
            info!("{} -> {}", lin_velocity.0, ang_velocity.0);
            lin_velocity.0 = Vec3::ZERO;
            ang_velocity.0 = Vec3::ZERO;
        }
    }

    // // Feed the jump action every frame as long as the player holds the jump button. If the player
    // // stops holding the jump button, simply stop feeding the action.
    // if keyboard.pressed(KeyCode::Space) {
    //     controller.action(ControlScheme::Jump(Default::default()));
    // }
}
