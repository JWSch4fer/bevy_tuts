use bevy::prelude::*;
use bevy::window::{ Window,  PrimaryWindow};
use rand::random;

use crate::enemy::components::Enemy;
use crate::enemy::{SAFE_DISTANCE};

use super::components::Star;
use super::resources::*;
use crate::star::{NUMBER_OF_STARS, MAX_ATTEMPTS};


pub fn spawn_stars(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.single().unwrap();
    // Collect enemy positions (2D)
    let enemy_positions: Vec<Vec2> =
        enemy_query.iter().map(|t| t.translation.truncate()).collect();

    for _ in 0..NUMBER_OF_STARS {
        let mut pos2d;
        let mut tries = 0;
        // Retry loop: pick until far enough or give up
        loop {
            pos2d = Vec2::new(
                random::<f32>() * window.width(),
                random::<f32>() * window.height(),
            );
            // Check distance to every enemy
            if enemy_positions
                .iter()
                .all(|&e_pos| e_pos.distance(pos2d) > SAFE_DISTANCE)
                || tries >= MAX_ATTEMPTS
            {
                break;
            }
            tries += 1;
        }

        commands.spawn((
            Sprite::from_image(asset_server.load("sprites/star.png")),
            Transform::from_xyz(pos2d.x, pos2d.y, 0.0),
            Star {},
        ));
    }
}

pub fn tick_star_spawn_timer(mut star_spawn_timer: ResMut<StarSpawnTimer>, time: Res<Time>) {
    star_spawn_timer.timer.tick(time.delta());
}

pub fn spawn_stars_over_time(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    star_spawn_timer: Res<StarSpawnTimer>,
) {
    if star_spawn_timer.timer.finished() {
        let window = window_query.single().unwrap();
        let pos2d = Vec2::new(
            random::<f32>() * window.width(),
            random::<f32>() * window.height(),
        );

        commands.spawn((
            Sprite::from_image(asset_server.load("sprites/star.png")),
            Transform::from_xyz(pos2d.x, pos2d.y, 0.0),
            Star {},
        ));
    }
}
