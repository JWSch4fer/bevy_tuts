use bevy::prelude::*;
use bevy::window::{Window, PrimaryWindow};

use super::components::Player;

use crate::enemy::components::Enemy;
use crate::enemy::{ENEMY_SIZE};

use crate::star::components::Star;
use crate::star::{STAR_SIZE};

use crate::score::resources::Score;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_SIZE: f32  = 64.0;

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

pub fn player_hit_enemy(
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

pub fn player_hit_star(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    star_query: Query<(Entity, &Transform), With<Star>>,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (star_entity, star_transform) in star_query.iter() {
            let distance = player_transform
                .translation
                .distance(star_transform.translation);

            if distance < PLAYER_SIZE / 2.0 + STAR_SIZE / 2.0 {
                info!("Player hit star!");
                score.value += 1;
                let sound_effect = asset_server.load("audio/laserLarge_001.ogg");
                commands.spawn((
                    AudioPlayer::new(sound_effect),
                    PlaybackSettings::ONCE,
                ));
                commands.entity(star_entity).despawn();
            }
        }
    }
}
