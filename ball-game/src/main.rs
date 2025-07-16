use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::window::{WindowPlugin, PresentMode, Window, WindowResolution, PrimaryWindow};
use std::num::NonZeroU32;
use rand::random;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_SIZE: f32  = 64.0;
const NUMBER_OF_ENEMIES: u16 = 4;
pub const ENEMY_SPEED: f32 = 200.0;
pub const ENEMY_SIZE: f32 = 64.0; // This is the enemy sprite size.
fn main() {

    App::new().add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(800.0, 800.0),
                    // Completely disable VSync (may allow tearing):
                    present_mode: PresentMode::AutoNoVsync,
                    // On Windows, limit GPU frame queue to 1 for lower latency:
                    desired_maximum_frame_latency: Some(NonZeroU32::new(3).unwrap()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(AssetPlugin{
                //note assets/ is the default directory that bevy looks in
                //for preprocessed assets
                //we can change this as needed
                //also note that with cross compilation can complicate things
                file_path: "assets".to_string(),
                ..Default::default()
            }),
            )
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_enemies)
        .add_systems(Update, (player_movement, enemy_movement)) //remove lag in movement 
        .add_systems(Update, (confine_player_movement, confine_enemy_movement ))
        .add_systems(Update, update_enemy_direction)
        .add_systems(Update, enemy_hit_player)
        .run();
}

#[derive(Component)]
pub struct Player {}

pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>
    ) {

    // there should only be one instance of window that is labeled
    // primary window as long as bevy is running.
    let window = window_query.single().expect("Primary Window not found?");


    commands.spawn((
        // Create a Sprite from an image handle
        Sprite::from_image(asset_server.load("sprites/ball_blue_large.png")),
        // Position at the window's center
        Transform::from_xyz(
            window.width() / 2.0,
            window.height() / 2.0,
            0.0,
        ),
        // Your marker component
        Player{},
    ));
}

#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2,
}

pub fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single().unwrap();

    for _ in 0..NUMBER_OF_ENEMIES {
        let random_x = random::<f32>() * window.width();
        let random_y = random::<f32>() * window.height();

        commands.spawn((
            //Create s sprite for the enemies
            Sprite::from_image(asset_server.load("sprites/ball_red_large.png")),
            Transform::from_xyz(random_x, random_y, 0.0),
            Enemy {direction: Vec2::new(random::<f32>(), random::<f32>()).normalize()},
        ));
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {

    let window = window_query.single().expect("Primary Window not found?");
  
    commands.spawn((
        Camera2d,
        Transform::from_xyz(
            window.width()  / 2.0,
            window.height() / 2.0,
            0.0,
        ),
    ));
}

pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    // `single_mut()` returns a Result; unwrap or handle errors as needed
    if let Ok(mut transform) = player_query.single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        // Use `delta_secs()` instead of the removed `delta_seconds()`
        transform.translation += direction * PLAYER_SPEED * time.delta_secs();
    }
}

// ---------------------------------------------------------------------------
// System: confine the player within window bounds
// ---------------------------------------------------------------------------
pub fn confine_player_movement(
    mut query: Query<&mut Transform, With<Player>>,   // Query::single_mut() for exclusive mut access :contentReference[oaicite:6]{index=6}
    window_query: Query<&Window, With<PrimaryWindow>>, // Query the primary window for dimensions :contentReference[oaicite:7]{index=7}
) {
    if let Ok(mut transform) = query.single_mut() {
        let window = window_query
            .single()
            .expect("PrimaryWindow not found");

        // Compute the allowed min/max positions
        let half = PLAYER_SIZE / 2.0;
        let x_min = half;
        let x_max = window.width()  - half;
        let y_min = half;
        let y_max = window.height() - half;

        // Clamp the translation to stay within the window
        let mut pos = transform.translation;
        pos.x = pos.x.clamp(x_min, x_max);
        pos.y = pos.y.clamp(y_min, y_max);

        transform.translation = pos;
    }
}

pub fn enemy_movement(mut enemy_query: Query<(&mut Transform, &Enemy)>, time: Res<Time>) {
    for (mut transform, enemy) in enemy_query.iter_mut() {
        let direction = Vec3::new(enemy.direction.x, enemy.direction.y, 0.0);
        transform.translation += direction * ENEMY_SPEED * time.delta_secs();
    }
}

pub fn update_enemy_direction(
    mut commands: Commands,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single().unwrap();

    let half_enemy_size = ENEMY_SIZE / 2.0; // 32.0
    let x_min = 0.0 + half_enemy_size;
    let x_max = window.width() - half_enemy_size;
    let y_min = 0.0 + half_enemy_size;
    let y_max = window.height() - half_enemy_size;

    for (transform, mut enemy) in enemy_query.iter_mut() {
        let mut direction_changed = false;

        let translation = transform.translation;
        if translation.x < x_min || translation.x > x_max {
            enemy.direction.x *= -1.0;
            direction_changed = true;
        }
        if translation.y < y_min || translation.y > y_max {
            enemy.direction.y *= -1.0;
            direction_changed = true;
        }

        // Play SFX
        if direction_changed {
            // Play Sound Effect
            // Load two variants and choose one at random
            let s1 = asset_server.load("audio/pluck_001.ogg");
            let s2 = asset_server.load("audio/pluck_002.ogg");
            let pick = if rand::random::<f32>() > 0.5 { s1 } else { s2 };

            // Spawn an entity to play the sound once
            commands.spawn((
                AudioPlayer::new(pick),
                PlaybackSettings::ONCE,
            ));
        }
    }
}

pub fn confine_enemy_movement(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().unwrap();

    let half_enemy_size = ENEMY_SIZE / 2.0;
    let x_min = 0.0 + half_enemy_size;
    let x_max = window.width() - half_enemy_size;
    let y_min = 0.0 + half_enemy_size;
    let y_max = window.height() - half_enemy_size;

    for mut transform in enemy_query.iter_mut() {
        let mut translation = transform.translation;

        // Bound the enemy x position
        if translation.x < x_min {
            translation.x = x_min;
        } else if translation.x > x_max {
            translation.x = x_max;
        }
        // Bound the enemy y position
        if translation.y < y_min {
            translation.y = y_min;
        } else if translation.y > y_max {
            translation.y = y_max;
        }

        transform.translation = translation;
    }
}

pub fn enemy_hit_player(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((player_entity, player_transform)) = player_query.single_mut() {
        for enemy_transform in enemy_query.iter() {
            let distance = player_transform
                .translation
                .distance(enemy_transform.translation);
            let player_radius = PLAYER_SIZE / 2.0;
            let enemy_radius = ENEMY_SIZE / 2.0;
            if distance < player_radius + enemy_radius {
                println!("Enemy hit player! Game Over!");
                let sound_effect = asset_server.load("audio/explosionCrunch_001.ogg");
                // Spawn an entity to play the sound once
                commands.spawn((
                    AudioPlayer::new(sound_effect),
                    PlaybackSettings::ONCE,
                ));
                commands.entity(player_entity).despawn();
            }
        }
    }
}
