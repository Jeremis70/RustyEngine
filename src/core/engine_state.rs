use std::time::{Duration, Instant};

pub struct EngineState {
    pub delta_time: Duration,
    pub total_time: Duration,
    pub frame_count: u64,
    pub fps: f64,

    // Internal
    last_frame: Instant,
    fps_update_timer: Duration,
    fps_frame_count: u64,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            delta_time: Duration::ZERO,
            total_time: Duration::ZERO,
            frame_count: 0,
            fps: 0.0,
            last_frame: Instant::now(),
            fps_update_timer: Duration::ZERO,
            fps_frame_count: 0,
        }
    }

    /// Update timing and FPS. Called once per frame before update callbacks.
    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_frame;
        self.last_frame = now;

        self.total_time += self.delta_time;
        self.frame_count += 1;

        // Update FPS every 500ms
        self.fps_update_timer += self.delta_time;
        self.fps_frame_count += 1;

        if self.fps_update_timer.as_secs_f64() >= 0.5 {
            self.fps = self.fps_frame_count as f64 / self.fps_update_timer.as_secs_f64();
            self.fps_update_timer = Duration::ZERO;
            self.fps_frame_count = 0;
        }
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn last_frame_instant(&self) -> Instant {
        self.last_frame
    }
}
