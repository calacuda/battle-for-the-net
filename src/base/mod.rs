// use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};

#[derive(Component)]
pub struct PlayerMeshMark;

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        info!("added base plugin.");
        app.add_systems(Startup, setup);
        app.add_systems(Update, camera_track_player);
        app.add_systems(Update, player_movement);
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

    let transform = Transform::from_xyz(0.0, 0.0, 0.0);

    commands.spawn((
        SceneRoot(asset_server.load("net-lv-01.glb#Scene0")),
        transform,
    ));
    commands.spawn((
        SceneRoot(asset_server.load("temp-char.glb#Scene0")),
        transform,
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

pub fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_transform: Single<Option<&mut Transform>, With<PlayerMeshMark>>,
    mut query: Query<&mut Projection, (With<Camera>, Without<PlayerMeshMark>)>,
) {
    let Some(player_transform) = player_transform.as_mut() else {
        warn!("player not found");
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= Vec3::new(1.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += Vec3::new(1.0, 0.0, 0.0);
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction -= Vec3::new(0.0, 0.0, 1.0);
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += Vec3::new(0.0, 0.0, 1.0);
    }

    player_transform.translation += time.delta_secs() * direction * 10.;

    for mut projection in query.iter_mut() {
        let Projection::Perspective(ortho) = &mut *projection else {
            continue;
        };

        if keyboard_input.pressed(KeyCode::KeyZ) {
            ortho.fov += 0.1;
        }

        if keyboard_input.pressed(KeyCode::KeyX) {
            ortho.fov -= 0.1;
        }

        if ortho.fov < 0.5 {
            ortho.fov = 0.5;
        }
    }
}
