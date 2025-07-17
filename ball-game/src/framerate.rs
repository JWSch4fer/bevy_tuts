use bevy::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use bevy::app::Last;                    // bring the `Last` schedule label into scope

#[derive(Resource)]
pub struct FrameLimiterState {
    /// Reference start time
    pub init: Instant,
    /// Desired frame interval (nanoseconds)
    pub target_ns: AtomicU64,
    /// “Memory target”: the timestamp (ns since `init`) when the *next* frame
    /// was supposed to start, based on the actual end of the last frame.
    pub memory_target_ns: AtomicU64,
}


pub struct FrameLimiterPlugin {
    pub target_fps: f64,
}

impl Default for FrameLimiterPlugin {
    fn default() -> Self {
        Self { target_fps: 60.0 }  // default to 60 FPS
    }
}

impl FrameLimiterPlugin {
    pub fn with_fps(fps: f64) -> Self {
        Self { target_fps: fps }
    }
}

impl Plugin for FrameLimiterPlugin {
    fn build(&self, app: &mut App) {
        // Compute nanoseconds per frame
        let fps = self.target_fps.max(1.0);
        let target_ns = (1_000_000_000.0 / fps) as u64;

        // Mark “now” as the zero point
        let now = Instant::now();
        // First memory target = one interval after start
        let first_target = target_ns;

        app.insert_resource(FrameLimiterState {
            init:               now,
            target_ns:          AtomicU64::new(target_ns),
            memory_target_ns:   AtomicU64::new(first_target),
        });

        app.add_systems(Last, (enforce_frame_rate,));
    }
}


fn enforce_frame_rate(state: Res<FrameLimiterState>) {
    // ─── Read clocks ───────────────────────────────────────────────────────
    let now_ns        = state.init.elapsed().as_nanos() as u64;
    let target_ns     = state.target_ns.load(Ordering::Relaxed);
    let memory_t_ns   = state.memory_target_ns.load(Ordering::Relaxed);

    // ─── Compute error: actual start vs. expected start ────────────────────
    // error = now_ns − memory_target_ns
    //   < 0 = early (we’ve come before the target)
    //   > 0 = late  (we’re behind the target)
    let error_ns_i = now_ns as i128 - memory_t_ns as i128;
    let error_ms   = error_ns_i as f64 / 1_000_000.0;
    info!(
        "Error: {:+} ns ({:+.3} ms) | now={} | mem_t={}",
        error_ns_i, error_ms, now_ns, memory_t_ns
    );

    // ─── Decide if we sleep ────────────────────────────────────────────────
    // We want to sleep until the *next* memory target, which is memory_t_ns + target_ns.
    // The time to wait = (memory_t_ns + target_ns) − now_ns = target_ns − error_ns
    let wait_ns = if error_ns_i < target_ns as i128 {
        // Only sleep if we are less than one interval behind
        (target_ns as i128 - error_ns_i).max(0) as u64
    } else {
        // error_ns_i >= target_ns : we’re too late—skip sleeping
        0
    };

    // ─── Sleep with spin_sleep’s hybrid OS+spin ────────────────────────────
    if wait_ns > 0 {
        spin_sleep::sleep(Duration::from_nanos(wait_ns));
    }

    // ─── Update memory_target for the *following* frame ───────────────────
    // Use the *actual* current time as the new baseline:
    // next_memory_target = now_ns + target_ns
    let next_mem = now_ns.saturating_add(target_ns);
    state.memory_target_ns.store(next_mem, Ordering::Relaxed);
}
