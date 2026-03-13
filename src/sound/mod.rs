use crate::{make_error_result, Error, Logger};
use kira::{
    effect::panning_control::PanningControlHandle,
    sound::{PlaybackState},
    Easing, Panning, StartTime, Tween,
    sound::static_sound::{StaticSoundHandle, StaticSoundData},
    AudioManager, AudioManagerSettings, DefaultBackend,
};
use std::{
    cell::RefCell,
    rc::Rc,
    default,
    ops::Range,
    time::Duration,
    path::{Path, PathBuf},
};

pub struct Sound {
    backend: Result<AudioManager, kira::backend::cpal::Error>,
    logger: Rc<RefCell<Logger>>,
}

impl Sound {
    pub(crate) fn new(logger: Rc<RefCell<Logger>>) -> Self {
        let backend = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default());
        match backend {
            Ok(b) => {
                return Self {
                    backend: Ok(b),
                    logger,
                }
            }
            Err(err) => {
                let msg = format!(
                    "Failed initializing audio backend, sound will not play: {}",
                    err
                );
                let instance = Self {
                    backend: Err(err),
                    logger,
                };
                instance.logger.borrow().log_error(msg);
                return instance;
            }
        }
    }

    pub fn play_sound(&mut self, resource: SoundResource) -> Result<SoundHandle, Error> {
        match &mut self.backend {
            Ok(backend) => {
                let play_result = backend.play(resource.val);
                match play_result {
                    Ok(handle) => return Ok(SoundHandle(handle)),
                    Err(err) => {
                        return make_error_result("Could not play sound", Some(Box::new(err)))
                    }
                }
            }
            Err(err) => {
                return make_error_result(
                    format!(
                        "Failed initialiazing audio backend, cannot play sound: {}",
                        err.to_string()
                    )
                    .as_str(),
                    None,
                )
            }
        };
    }
}

fn new_tween(duration: Duration) -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration,
        easing: Easing::Linear,
    }
}

fn new_none_tween() -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration: Duration::from_micros(1),
        easing: Easing::Linear,
    }
}

pub struct SoundHandle(pub(crate) StaticSoundHandle);
impl SoundHandle {
    const MIN_PANNING: f32 = -1.0;
    const MAX_PANNING: f32 = 1.0;

    pub fn pause(&mut self) {
        self.0.pause(new_none_tween());
    }

    pub fn pause_ease(&mut self, tween: Duration) {
        self.0.pause(new_tween(tween))
    }

    pub fn get_position_seconds(&self) -> f64 {
        self.0.position()
    }

    pub fn resume(&mut self) {
        self.0.resume(new_none_tween())
    }

    pub fn resume_ease(&mut self, tween: Duration) {
        self.0.resume(new_tween(tween))
    }

    pub fn seek_by(&mut self, seconds: f64) {
        self.0.seek_by(seconds)
    }

    pub fn seek_to(&mut self, seconds: f64) {
        self.0.seek_to(seconds)
    }

    pub fn stop(&mut self) {
        self.0.stop(new_none_tween())
    }

    pub fn stop_ease(&mut self, tween: Duration) {
        self.0.stop(new_tween(tween))
    }

    pub fn set_volume(&mut self, decibels: f32) {
        self.0.set_volume(decibels, new_none_tween());
    }

    pub fn set_volume_ease(&mut self, decibels: f32, tween: Duration) {
        self.0.set_volume(decibels, new_tween(tween));
    }

    pub fn loop_sound(&mut self) {
        self.0.set_loop_region(0.0..);
    }

    pub fn loop_sound_region(&mut self, range_seconds: Range<f64>) {
        self.0.set_loop_region(range_seconds)
    }

    pub fn set_panning(&mut self, panning: f32) {
        self.set_panning_ease(panning, Duration::from_micros(1));
    }

    pub fn set_panning_ease(&mut self, panning: f32, tween: Duration) {
        let mut resolved_panning = panning;
        if panning < Self::MIN_PANNING {
            resolved_panning = Self::MIN_PANNING
        } else if panning > Self::MAX_PANNING {
            resolved_panning = Self::MAX_PANNING
        }

        self.0.set_panning(resolved_panning, new_tween(tween));
    }
}

#[derive(Clone)]
pub struct SoundResource {
    pub (crate) val: StaticSoundData,
}

impl SoundResource {

    pub fn from_path<P>(path: P) -> Result<SoundResource, Error> where P: AsRef<Path> {
        let sound_data_result = StaticSoundData::from_file(&path);
        match sound_data_result {
            Ok(sound_data) => {
                return Ok(SoundResource {
                    val: sound_data
                })    
            },
            Err(err) => return make_error_result(format!("Could not create sound resource from specified path. File might be missing, corrupted, or in a wrong format").as_str(), Some(Box::new(err)))
        }
    }

}
