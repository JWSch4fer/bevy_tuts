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
        .add_systems(PreUpdate, player_movement) //remove lag in movement 
        .add_systems(Update, (confine_player_movement,  ))
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
pub struct Enemy {}

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
            Enemy {},
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
