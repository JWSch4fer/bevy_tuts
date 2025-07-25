// mod framerate;  //load framerate.rs
pub mod events;
mod systems;

pub mod enemy;
mod player;
pub mod score;
pub mod star;

use events::*;
use systems::*;

use enemy::EnemyPlugin;
use player::PlayerPlugin;
use score::ScorePlugin;
use star::StarPlugin;

use bevy::prelude::*;


// use bevy::input::ButtonInput;
use bevy::window::{WindowPlugin, PresentMode, Window, WindowResolutionsys;
use std::num::NonZeroU32;
// use rand::random;
// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};
// use framerate::FrameLimiterPlugin;

//use framerate::{FramepacePlugin, FramepaceSettings, Limiter};
// ensure stars aren't too close to a spawned enemy
//define time between star spawns
// const STAR_SPAWN_TIME: f32 = 1.0;

// run every frame (or less often if you like)
// ---------------------------------------------------------------------------


// run every frame (or less often if you like)
// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------
// fn fps_system(diagnostics: Res<DiagnosticsStore>) {
//     if let Some(fps) = diagnostics
//         .get(&FrameTimeDiagnosticsPlugin::FPS)
//         .and_then(|d| d.average())
//     {
//         info!("FPS: {:.2}", fps);
//     }
// }

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
        .add_event::<GameOver>()
        .add_plugins(EnemyPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ScorePlugin)
        .add_plugins(StarPlugin)
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, exit_game)
        .add_systems(Update, handle_game_over)
        //.init_resource::<Score>()
        //.init_resource::<StarSpawnTimer>()
        //.add_plugins(FrameTimeDiagnosticsPlugin::default())
        //// cap framerate at 120fps
        ////.add_plugins(FrameLimiterPlugin::with_fps(120.0))
        //.add_systems(Startup, spawn_camera)
        //.add_systems(Startup, spawn_player)
        //.add_systems(Startup, spawn_enemies)
        //.add_systems(Startup, spawn_stars.after(spawn_enemies))
        //.add_systems(Update, (player_movement, enemy_movement)) //remove lag in movement 
        //// changing enemy direction has to come before confinment
        ////.add_systems(Update, (update_enemy_direction, enemy_hit_star))
        //.add_systems(Update, (update_enemy_direction, enemy_hit_star).after(enemy_movement).before(confine_enemy_movement))
        //.add_systems(Update, (confine_player_movement, confine_enemy_movement ))
        //.add_systems(Update, enemy_hit_player)
        //// .add_systems(Update, fps_system)
        //.add_systems(Update, player_hit_star)
        //.add_systems(Update, update_score)
        //.add_systems(Update, spawn_stars_over_time)
        //.add_systems(Update, tick_star_spawn_timer)
//        .insert_resource(FramepaceSettings {
//            limiter: Limiter::from_framerate(120.0),
//        })
        .run();
}














