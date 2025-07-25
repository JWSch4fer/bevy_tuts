use bevy::prelude::*;
use bevy::window::{ Window,  PrimaryWindow};
use rand::random;

use super::components::*;
use super::resources::*;
use super::{ENEMY_SIZE, ENEMY_SPEED, NUMBER_OF_ENEMIES};

use crate::star::{STAR_SIZE, components::Star};


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



pub fn enemy_hit_star(
    star_query: Query<&Transform, (With<Star>, Without<Enemy>)>,
    mut enemy_query: Query< (&Transform, &mut Enemy), (With<Enemy>, Without<Star>)>,
) {
    for star_transform in star_query.iter() {

        let star_xy = star_transform.translation.truncate();

        for (enemy_transform, mut enemy) in enemy_query.iter_mut() {
            let distance = star_transform
                .translation
                .distance(enemy_transform.translation);
            let enemy_xy = enemy_transform.translation.truncate();
            let delta_xy = star_xy - enemy_xy;

            let star_radius = STAR_SIZE / 2.0;
            let enemy_radius = ENEMY_SIZE / 2.0;

            //logic for an elastic collision
            if distance <= star_radius + enemy_radius {
                //enemy.direction *= Vec2::new(-1.0_f32, -1.0_f32);
                // normalize collision normal (skip zero‑length just in case)
                if let Some(normal) = delta_xy.try_normalize() {
                    // current velocity (unit) of the enemy
                    let v = enemy.direction;
                    // reflect: v' = v − 2(v·n)n
                    let reflected = v - 2.0 * v.dot(normal) * normal;
                    // write back (normalized to preserve speed)
                    enemy.direction = reflected.normalize(); 
                }  
            }
        }
    }
}

pub fn tick_enemy_spawn_timer(mut enemy_spawn_timer: ResMut<EnemySpawnTimer>, time: Res<Time>) {
    enemy_spawn_timer.timer.tick(time.delta());
}

pub fn spawn_enemies_over_time(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    enemy_spawn_timer: Res<EnemySpawnTimer>,
) {
    if enemy_spawn_timer.timer.finished() {
        let window = window_query.single().unwrap();

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
