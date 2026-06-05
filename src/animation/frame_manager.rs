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
    /// For single-frame animations (frame_count <= 1), always returns false
    /// but keeps the deadline fresh so WaitUntil sleeps instead of busy-looping.
    pub fn update(&mut self) -> bool {
        if self.frame_count <= 1 {
            self.last_frame_time = Instant::now();
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
    /// A `frame_count` of 0 is treated as 1 (single-frame placeholder).
    pub fn set_animation(&mut self, frame_count: u32, fps: f32) {
        let count = frame_count.max(1);
        if self.frame_count != count {
            self.frame_count = count;
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
