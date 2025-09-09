use crate::{make_error_result, Error};
pub use bitcode::{decode, encode};
use bitcode::{DecodeOwned, Encode};
use std::{
    env,
    fs::{create_dir_all, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    result::Result,
};

/// Provides an interface to simplify saving game state
pub struct SaveManager {}

impl SaveManager {
    /// Used if [SaveManager::save] or [SaveManager::load] are called without a file name

    /// Returns a new [SaveManager] instance
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Saves the given content in binary format to the given path
    pub(crate) fn save<T, P>(&self, path: P, content: T) -> Result<(), Error>
    where
        T: Encode + DecodeOwned,
        P: AsRef<Path>,
    {
        let create_dir_result = create_dir_all(&path);
        match create_dir_result {
            Ok(_) => {}
            Err(err) => {
                return make_error_result(
                    "Could not create target directories",
                    Some(Box::new(err)),
                )
            }
        }
        let file_result = File::create(&path);
        match file_result {
            Ok(mut file) => {
                let encoded = encode(&content);
                let result = file.write_all(&encoded);
                match result {
                    Ok(_) => return Ok(()),
                    Err(err) => {
                        return make_error_result("Failed writing save data", Some(Box::new(err)))
                    }
                }
            }
            Err(err) => return make_error_result("Failed opening file", Some(Box::new(err))),
        }
    }

    /// Saves the given content in binary format to the default path
    ///
    /// Windows: ``%APPDATA%/wgpu-rs-demo/save0001.bin``
    ///
    /// MacOS: ``~/Library/Application Support/wgpu-rs-demo/save0001.bin``
    ///
    /// Linux: ``~/.local/share/wgpu-rs-demo/save0001.bin``
    pub(crate) fn save_default<T>(&self, content: T) -> Result<(), Error>
    where
        T: Encode + DecodeOwned,
    {
        self.save(Self::get_default_path(), content)
    }

    /// Tries to load content of a binary file into an instance of type ``T``
    pub(crate) fn load<T, P>(&self, path: P) -> Result<T, Error>
    where
        T: Encode + DecodeOwned,
        P: AsRef<Path>,
    {
        let mut buf: Vec<u8> = Vec::new();
        let file_result = File::open(&path);
        match file_result {
            Ok(mut file) => {
                let result = file.read_to_end(&mut buf);
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        return make_error_result("Failed reading save data", Some(Box::new(err)))
                    }
                }
                match file {
                    _ => {}
                }
                let decoded = decode::<T>(&mut buf);
                match decoded {
                    Ok(decoded) => {
                        return Ok(decoded);
                    }
                    Err(err) => {
                        return make_error_result("Failed decoding save data", Some(Box::new(err)))
                    }
                }
            }
            Err(err) => return make_error_result("Failed opening file", Some(Box::new(err))),
        }
    }

    /// Tries to load content of a binary file into an instance of type ``T`` from default path
    ///
    /// Windows: ``%APPDATA%/exlib/save0001.bin``
    ///
    /// MacOS: ``~/Library/Application Support/exlib/save0001.bin``
    ///
    /// Linux: ``~/.local/share/exlib/save0001.bin``
    pub(crate) fn load_default<T>(&self) -> Result<T, Error>
    where
        T: Encode + DecodeOwned,
    {
        self.load(Self::get_default_path())
    }

    /// Returns the default save path prefix for Windows
    #[cfg(target_os = "windows")]
    pub fn get_default_path() -> PathBuf {
        let appdata = env::var("APPDATA");
        match appdata {
            Ok(a) => {
                return PathBuf::from(format!(r"{}\wgpu-rs-demo\save0001.bin", a));
            }
            Err(_) => return PathBuf::from(r".\"),
        }
    }

    /// Returns the default save path prefix for MacOS
    #[cfg(target_os = "macos")]
    pub fn get_default_path() -> PathBuf {
        let home = env::var("HOME");
        match home {
            Ok(h) => {
                return PathBuf::from(format!(
                    "{}/Library/Application Support/wgpu-rs-demo/save0001.bin",
                    h
                ))
            }
            Err(_) => return PathBuf::from("./"),
        }
    }

    /// Returns the default save path prefix for Linux
    #[cfg(target_os = "linux")]
    pub fn get_default_path() -> PathBuf {
        let home = env::var("HOME");
        match home {
            Ok(h) => return PathBuf::from(format!("{}/.local/share/wgpu-rs-demo/save0001.bin", h)),
            Err(_) => return PathBuf::from("./"),
        }
    }
}
