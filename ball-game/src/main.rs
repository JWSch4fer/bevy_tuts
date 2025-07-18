// mod framerate;  //load framerate.rs

use bevy::prelude::*;
use bevy::input::ButtonInput;
use bevy::window::{WindowPlugin, PresentMode, Window, WindowResolution, PrimaryWindow};
use std::num::NonZeroU32;
use rand::random;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};
// use framerate::FrameLimiterPlugin;

//use framerate::{FramepacePlugin, FramepaceSettings, Limiter};
// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_SIZE: f32  = 64.0;

const NUMBER_OF_ENEMIES: u16 = 4;
const ENEMY_SPEED: f32 = 200.0;
const ENEMY_SIZE: f32 = 64.0; // This is the enemy sprite size.

// ensure stars aren't too close to a spawned enemy
const NUMBER_OF_STARS: u16 = 10; // This is the enemy sprite size.
const STAR_SIZE: f32  = 30.0;
const SAFE_DISTANCE: f32 = ENEMY_SIZE + STAR_SIZE + 20.0;  // e.g. leave a 20px margin
const MAX_ATTEMPTS: usize = 11;

//define time between star spawns
const STAR_SPAWN_TIME: f32 = 1.0;

// run every frame (or less often if you like)
// ---------------------------------------------------------------------------


// run every frame (or less often if you like)
// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------
fn fps_system(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.average())
    {
        info!("FPS: {:.2}", fps);
    }
}

// ---------------------------------------------------------------------------
// Start the application
// ---------------------------------------------------------------------------
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
        .init_resource::<Score>()
        .init_resource::<StarSpawnTimer>()
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // cap framerate at 120fps
        //.add_plugins(FrameLimiterPlugin::with_fps(120.0))
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_enemies)
        .add_systems(Startup, spawn_stars.after(spawn_enemies))
        .add_systems(Update, (player_movement, enemy_movement)) //remove lag in movement 
        // changing enemy direction has to come before confinment
        //.add_systems(Update, (update_enemy_direction, enemy_hit_star))
        .add_systems(Update, (update_enemy_direction, enemy_hit_star).after(enemy_movement).before(confine_enemy_movement))
        .add_systems(Update, (confine_player_movement, confine_enemy_movement ))
        .add_systems(Update, enemy_hit_player)
        // .add_systems(Update, fps_system)
        .add_systems(Update, player_hit_star)
        .add_systems(Update, update_score)
        .add_systems(Update, spawn_stars_over_time)
        .add_systems(Update, tick_star_spawn_timer)
//        .insert_resource(FramepaceSettings {
//            limiter: Limiter::from_framerate(120.0),
//        })
        .run();
}

#[derive(Resource, Default)]
pub struct Score {
    pub value: u32,
}

// #[derive(Default)]
// pub struct Score {
//     value: 0,
// }
// impl Default for Score {
//     fn default() -> Score {
//         Score { value: 0 }
//     }
// }

#[derive(Component)]
pub struct Star {}

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

//pub fn spawn_stars(
//    mut commands: Commands,
//    window_query: Query<&Window, With<PrimaryWindow>>,
//    asset_server: Res<AssetServer>
//    ) {
//
//    // there should only be one instance of window that is labeled
//    // primary window as long as bevy is running.
//    let window = window_query.single().expect("Primary Window not found?");
//
//    for _ in 0..NUMBER_OF_STARS {
//        let random_x = random::<f32>() * window.width();
//        let random_y = random::<f32>() * window.height();
//
//        commands.spawn((
//            //Create s sprite for the enemies
//            Sprite::from_image(asset_server.load("sprites/star.png")),
//            Transform::from_xyz(random_x, random_y, 0.0),
//            Star {},
//        ));
//    }
//}

#[derive(Resource)]
pub struct StarSpawnTimer {
    pub timer: Timer,
}

impl Default for StarSpawnTimer {
    fn default() -> StarSpawnTimer {
        StarSpawnTimer {
            timer: Timer::from_seconds(STAR_SPAWN_TIME, TimerMode::Repeating),
        }
    }
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

pub fn update_score(score: Res<Score>) {
    if score.is_changed() {
        info!("Score: {}", score.value);
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
