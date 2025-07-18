use bevy::prelude::*;
use bevy::app::Last;
use spin_sleep::sleep;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Holds timing and pacing state for frame limiting.
#[derive(Resource)]
pub struct FrameLimiterState {
    /// Reference start time for high-precision elapsed.
    init: Instant,
    /// Desired frame interval in nanoseconds (1e9 / FPS).
    target_ns: AtomicU64,
    /// "Memory target": timestamp (ns since `init`) when the next frame should start.
    memory_target_ns: AtomicU64,
    /// Timestamp (ns since `init`) when the previous frame began.
    last_frame_ns: AtomicU64,
}

/// A Bevy plugin that caps the frame rate by sleeping the main thread.
pub struct FrameLimiterPlugin {
    /// Desired frames per second.
    pub target_fps: f64,
}

impl Default for FrameLimiterPlugin {
    fn default() -> Self {
        Self { target_fps: 60.0 }
    }
}

impl FrameLimiterPlugin {
    /// Create the plugin with a custom FPS cap.
    pub fn with_fps(fps: f64) -> Self {
        Self { target_fps: fps }
    }
}

impl Plugin for FrameLimiterPlugin {
    fn build(&self, app: &mut App) {
        // Compute nanoseconds per frame from FPS.
        let fps = self.target_fps.max(1.0);
        let target_ns = (1_000_000_000.0 / fps) as u64;

        // Mark "now" as the zero point.
        let now = Instant::now();
        // First memory target = one interval after start.
        let first_target = target_ns;

        app.insert_resource(FrameLimiterState {
            init: now,
            target_ns: AtomicU64::new(target_ns),
            memory_target_ns: AtomicU64::new(first_target),
            last_frame_ns: AtomicU64::new(0),
        });

        // Add the frame limiter system to run at the end of each frame.
        app.add_systems(Last, (enforce_frame_rate,));
    }
}

/// Enforces the frame cap by sleeping the thread and logs instantaneous FPS.
fn enforce_frame_rate(state: Res<FrameLimiterState>) {
    // 1) Absolute current time (ns since init).
    let now_ns = state.init.elapsed().as_nanos() as u64;
    
    // 2) Instantaneous FPS calculation.
    let prev_ns = state.last_frame_ns.swap(now_ns, Ordering::Relaxed);
    if prev_ns != 0 {
        let dt_ns = now_ns.saturating_sub(prev_ns);
        let fps = 1_000_000_000.0 / (dt_ns as f64);
        info!("Instant FPS: {:.2}", fps);
    }

    // 3) Frame pacing: compute error vs memory target.
    let target_ns = state.target_ns.load(Ordering::Relaxed);
    let mem_t_ns = state.memory_target_ns.load(Ordering::Relaxed);
    let error_ns_i = now_ns as i128 - mem_t_ns as i128;

    // How long to wait = target_ns - error_ns_i (clamped ≥ 0).
    let wait_ns = (target_ns as i128 - error_ns_i).max(0) as u64;
    
    // Log error and wait time if needed.
    info!("Frame Error: {:+} ns | wait: {} ns", error_ns_i, wait_ns);

    // 4) Sleep the remaining time using spin_sleep's hybrid OS+spin.
    if wait_ns > 0 {
        sleep(Duration::from_nanos(wait_ns));
    }

    // 5) Schedule next memory target based on *actual* now timestamp.
    let next_mem = now_ns.saturating_add(target_ns);
    state.memory_target_ns.store(next_mem, Ordering::Relaxed);
}
