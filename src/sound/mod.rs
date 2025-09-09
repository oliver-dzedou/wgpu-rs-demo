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

/// Provides an interface to the sound module of ``exlib``
pub struct Sound {
    backend: Result<AudioManager, kira::backend::cpal::Error>,
    logger: Rc<RefCell<Logger>>,
}

impl Sound {
    /// Creates a new [Sound] instance
    ///
    /// The sound backend initialization can fail. Since games can run without sound, the program does not panic.
    ///
    /// Instead an Err will be given when trying to play sound.
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

    /// Plays a sound
    ///
    /// Returns a [handle](crate::SoundHandle) that allows advanced
    /// playback options, pausing, resuming, stopping, etc...
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

/// Creates a new immediate linear tween from the given duration
fn new_tween(duration: Duration) -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration,
        easing: Easing::Linear,
    }
}

/// Create a new immediate linear tween with 1 microsecond duration, is equal to no tween
fn new_none_tween() -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration: Duration::from_micros(1),
        easing: Easing::Linear,
    }
}

/// Returned from [crate::Sound::play_sound]
///
/// Used to control the played sound
pub struct SoundHandle(pub(crate) StaticSoundHandle);
impl SoundHandle {
    const MIN_PANNING: f32 = -1.0;
    const MAX_PANNING: f32 = 1.0;

    /// Pauses the played sound
    ///
    /// Can be resumed with [crate::SoundHandle::resume] or [crate::SoundHandle::resume_ease]
    pub fn pause(&mut self) {
        self.0.pause(new_none_tween());
    }

    /// Eases the played sound into a pause
    ///
    /// Can be resumed with [crate::SoundHandle::resume] or [crate::SoundHandle::resume_ease]
    pub fn pause_ease(&mut self, tween: Duration) {
        self.0.pause(new_tween(tween))
    }

    /// Returns the current playback position of the sound in seconds
    pub fn get_position_seconds(&self) -> f64 {
        self.0.position()
    }

    /// Resumes a sound paused with [crate::SoundHandle::pause] or [crate::SoundHandle::pause_ease]
    pub fn resume(&mut self) {
        self.0.resume(new_none_tween())
    }

    /// Eases a sound paused with [crate::SoundHandle::pause] or [crate::SoundHandle::pause_ease] back into playing
    pub fn resume_ease(&mut self, tween: Duration) {
        self.0.resume(new_tween(tween))
    }

    /// Moves the playback position back by the specified amount of seconds
    pub fn seek_by(&mut self, seconds: f64) {
        self.0.seek_by(seconds)
    }

    /// Moves the playback position to the specified second
    pub fn seek_to(&mut self, seconds: f64) {
        self.0.seek_to(seconds)
    }

    /// Stops the sound. Cannot be resumed.
    pub fn stop(&mut self) {
        self.0.stop(new_none_tween())
    }

    /// Eases the sound into stopping. Cannot be resumed.
    pub fn stop_ease(&mut self, tween: Duration) {
        self.0.stop(new_tween(tween))
    }

    /// Sets the volume of the sound in decibels
    pub fn set_volume(&mut self, decibels: f32) {
        self.0.set_volume(decibels, new_none_tween());
    }

    /// Smoothly transitions the sound into the given volume in decibels
    pub fn set_volume_ease(&mut self, decibels: f32, tween: Duration) {
        self.0.set_volume(decibels, new_tween(tween));
    }

    /// Loops the sound
    pub fn loop_sound(&mut self) {
        self.0.set_loop_region(0.0..);
    }

    /// Loops the sound in the given range in seconds
    pub fn loop_sound_region(&mut self, range_seconds: Range<f64>) {
        self.0.set_loop_region(range_seconds)
    }

    /// Sets the panning of the sound
    ///
    /// ``-1`` is hard left, ``0`` is center and ``1`` is hard right
    ///
    /// Values not between ``-1`` and ``1`` will be clamped
    pub fn set_panning(&mut self, panning: f32) {
        self.set_panning_ease(panning, Duration::from_micros(1));
    }

    /// Smoothly transitions the panning of the sound
    ///
    /// ``-1`` is hard left, ``0`` is center and ``1`` is hard right
    ///
    /// Values not between ``-1`` and ``1`` will be clamped
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

/// Encapsulates sound data
/// 
/// Hold on to these resources and re-use them when playing, to avoid unneeded I/O operations
///
/// Cam be cheaply cloned, as the audio data is shared between all clones
///
/// Create using [crate::SoundResource::from_path]
///
/// Play using [crate::Sound::play_sound]
///
/// Supports ``mp3``, ``ogg``, ``flac`` and ``wav`` 
/// 
/// Be aware that sound decoding takes a long time, so it's recommended to instantiate your sound resources on game/level start up, rather than when you intend to play them
/// 
/// Also be aware that sound decoding is much faster in release mode, but the above still applies
#[derive(Clone)]
pub struct SoundResource {
    pub (crate) val: StaticSoundData,
}

impl SoundResource {

    /// Creates a new [SoundResource] from given path
    /// 
    /// Supports ``mp3``, ``ogg``, ``flac`` and ``wav`` 
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
