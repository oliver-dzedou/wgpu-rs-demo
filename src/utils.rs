use std::time::{Duration, Instant};

/// Provides various utility features that don't fit in other places
pub struct Utils {
    frame_count: i32,
    started_at: Option<Instant>,
    previous_frame: Option<Instant>,
}

impl Utils {
    /// Create a new [Utils] instance
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            started_at: None,
            previous_frame: None,
        }
    }

    /// Sets the [Instant] at which the game started
    pub(crate) fn set_started_at(&mut self, started_at: Instant) {
        self.started_at = Some(started_at);
    }

    /// Adds 1 to [Self::frame_count]
    pub(crate) fn increment_frames(&mut self) {
        self.frame_count = self.frame_count + 1;
    }

    /// Sets the [Instant] at which the previous frame occurred
    pub(crate) fn set_previous_frame(&mut self, previous_frame: Instant) {
        self.previous_frame = Some(previous_frame);
    }

    /// Returns the current FPS
    pub (crate) fn get_fps(&self) -> f32 {
        match self.started_at {
            Some(started_at) => {
                let elapsed_time = started_at.elapsed().as_secs_f32();
                if elapsed_time == 0.0 {
                    return 0.0;
                } else {
                    return self.frame_count as f32 / elapsed_time;
                };
            }
            None => return 0.0,
        }
    }

    /// Returns ``delta``, which represents the time since previous frame
    pub (crate) fn get_delta(&self) -> Duration {
        match self.previous_frame {
            Some(previous_frame) => {
                return previous_frame.elapsed();
            }
            None => {
                return Duration::new(0, 0);
            }
        }
    }
}
