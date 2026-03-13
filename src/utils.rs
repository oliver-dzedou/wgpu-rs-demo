use std::time::{Duration, Instant};

pub struct Utils {
    frame_count: i32,
    started_at: Option<Instant>,
    previous_frame: Option<Instant>,
}

impl Utils {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            started_at: None,
            previous_frame: None,
        }
    }

    pub(crate) fn set_started_at(&mut self, started_at: Instant) {
        self.started_at = Some(started_at);
    }

    pub(crate) fn increment_frames(&mut self) {
        self.frame_count = self.frame_count + 1;
    }

    pub(crate) fn set_previous_frame(&mut self, previous_frame: Instant) {
        self.previous_frame = Some(previous_frame);
    }

    pub(crate) fn get_fps(&self) -> f32 {
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

    pub(crate) fn get_delta(&self) -> Duration {
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
