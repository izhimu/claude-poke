use std::time::{Duration, Instant};

/// Manages animation frame timing.
pub struct FrameManager {
    current_frame: u32,
    frame_count: u32,
    frame_duration: Duration,
    last_frame_time: Instant,
}

impl FrameManager {
    /// Create a new frame manager.
    /// `frame_count`: total number of frames in the animation.
    /// `fps`: frames per second for this animation.
    pub fn new(frame_count: u32, fps: f32) -> Self {
        let frame_duration = if fps > 0.0 {
            Duration::from_secs_f32(1.0 / fps)
        } else {
            Duration::from_millis(100)
        };

        Self {
            current_frame: 0,
            frame_count,
            frame_duration,
            last_frame_time: Instant::now(),
        }
    }

    /// Update the animation. Returns true if the frame changed.
    pub fn update(&mut self) -> bool {
        if self.frame_count <= 1 {
            return false;
        }

        let now = Instant::now();
        if now.duration_since(self.last_frame_time) >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frame_count;
            self.last_frame_time = now;
            true
        } else {
            false
        }
    }

    /// Get the current frame index.
    pub fn current_frame(&self) -> u32 {
        self.current_frame
    }

    /// Get the current frame count.
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Returns the Instant when the next frame should be rendered.
    pub fn next_frame_deadline(&self) -> Instant {
        self.last_frame_time + self.frame_duration
    }

    /// Reset to frame 0.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_frame_time = Instant::now();
    }

    /// Change animation parameters (e.g., when state changes).
    pub fn set_animation(&mut self, frame_count: u32, fps: f32) {
        if self.frame_count != frame_count {
            self.frame_count = frame_count;
            self.current_frame = 0;
        }
        self.frame_duration = if fps > 0.0 {
            Duration::from_secs_f32(1.0 / fps)
        } else {
            Duration::from_millis(100)
        };
        self.last_frame_time = Instant::now();
    }
}
