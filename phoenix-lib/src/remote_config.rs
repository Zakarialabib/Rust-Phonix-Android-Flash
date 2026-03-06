//! Remote control configuration for Android TV boxes
//!
//! Provides IR remote configuration database and generation utilities
//! for remote.conf (Amlogic) and .kl keylayout (Android) files.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use std::path::Path;

/// Linux input key codes for IR remote mapping
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LinuxKeyCode {
    KEY_POWER = 116,
    KEY_MUTE = 113,
    KEY_VOLUMEUP = 115,
    KEY_VOLUMEDOWN = 114,
    KEY_UP = 103,
    KEY_DOWN = 108,
    KEY_LEFT = 105,
    KEY_RIGHT = 106,
    KEY_OK = 352,
    KEY_ENTER = 28,
    KEY_BACK = 158,
    KEY_HOME = 102,
    KEY_MENU = 139,
    KEY_INFO = 358,
    KEY_CHANNELUP = 402,
    KEY_CHANNELDOWN = 403,
    KEY_1 = 2,
    KEY_2 = 3,
    KEY_3 = 4,
    KEY_4 = 5,
    KEY_5 = 6,
    KEY_6 = 7,
    KEY_7 = 8,
    KEY_8 = 9,
    KEY_9 = 10,
    KEY_0 = 11,
    KEY_RED = 398,
    KEY_GREEN = 399,
    KEY_YELLOW = 400,
    KEY_BLUE = 401,
    KEY_PLAY = 207,
    KEY_PAUSE = 119,
    KEY_STOP = 128,
    KEY_REWIND = 168,
    KEY_FASTFORWARD = 208,
    KEY_RECORD = 167,
    KEY_PREVIOUS = 165,
    KEY_NEXT = 163,
    KEY_SUBTITLE = 370,
    KEY_AUDIO = 392,
    KEY_TV = 377,
    KEY_EPG = 365,
    KEY_SLEEP = 142,
    KEY_SETUP = 141,
    KEY_FAVORITES = 364,
}

impl LinuxKeyCode {
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }

    pub fn name(&self) -> &'static str {
        match self {
            LinuxKeyCode::KEY_POWER => "KEY_POWER",
            LinuxKeyCode::KEY_MUTE => "KEY_MUTE",
            LinuxKeyCode::KEY_VOLUMEUP => "KEY_VOLUMEUP",
            LinuxKeyCode::KEY_VOLUMEDOWN => "KEY_VOLUMEDOWN",
            LinuxKeyCode::KEY_UP => "KEY_UP",
            LinuxKeyCode::KEY_DOWN => "KEY_DOWN",
            LinuxKeyCode::KEY_LEFT => "KEY_LEFT",
            LinuxKeyCode::KEY_RIGHT => "KEY_RIGHT",
            LinuxKeyCode::KEY_OK => "KEY_OK",
            LinuxKeyCode::KEY_ENTER => "KEY_ENTER",
            LinuxKeyCode::KEY_BACK => "KEY_BACK",
            LinuxKeyCode::KEY_HOME => "KEY_HOME",
            LinuxKeyCode::KEY_MENU => "KEY_MENU",
            LinuxKeyCode::KEY_INFO => "KEY_INFO",
            LinuxKeyCode::KEY_CHANNELUP => "KEY_CHANNELUP",
            LinuxKeyCode::KEY_CHANNELDOWN => "KEY_CHANNELDOWN",
            LinuxKeyCode::KEY_1 => "KEY_1",
            LinuxKeyCode::KEY_2 => "KEY_2",
            LinuxKeyCode::KEY_3 => "KEY_3",
            LinuxKeyCode::KEY_4 => "KEY_4",
            LinuxKeyCode::KEY_5 => "KEY_5",
            LinuxKeyCode::KEY_6 => "KEY_6",
            LinuxKeyCode::KEY_7 => "KEY_7",
            LinuxKeyCode::KEY_8 => "KEY_8",
            LinuxKeyCode::KEY_9 => "KEY_9",
            LinuxKeyCode::KEY_0 => "KEY_0",
            LinuxKeyCode::KEY_RED => "KEY_RED",
            LinuxKeyCode::KEY_GREEN => "KEY_GREEN",
            LinuxKeyCode::KEY_YELLOW => "KEY_YELLOW",
            LinuxKeyCode::KEY_BLUE => "KEY_BLUE",
            LinuxKeyCode::KEY_PLAY => "KEY_PLAY",
            LinuxKeyCode::KEY_PAUSE => "KEY_PAUSE",
            LinuxKeyCode::KEY_STOP => "KEY_STOP",
            LinuxKeyCode::KEY_REWIND => "KEY_REWIND",
            LinuxKeyCode::KEY_FASTFORWARD => "KEY_FASTFORWARD",
            LinuxKeyCode::KEY_RECORD => "KEY_RECORD",
            LinuxKeyCode::KEY_PREVIOUS => "KEY_PREVIOUS",
            LinuxKeyCode::KEY_NEXT => "KEY_NEXT",
            LinuxKeyCode::KEY_SUBTITLE => "KEY_SUBTITLE",
            LinuxKeyCode::KEY_AUDIO => "KEY_AUDIO",
            LinuxKeyCode::KEY_TV => "KEY_TV",
            LinuxKeyCode::KEY_EPG => "KEY_EPG",
            LinuxKeyCode::KEY_SLEEP => "KEY_SLEEP",
            LinuxKeyCode::KEY_SETUP => "KEY_SETUP",
            LinuxKeyCode::KEY_FAVORITES => "KEY_FAVORITES",
        }
    }
}

/// Android key code mapping for .kl files
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum AndroidKeyCode {
    DPAD_UP,
    DPAD_DOWN,
    DPAD_LEFT,
    DPAD_RIGHT,
    DPAD_CENTER,
    BACK,
    HOME,
    MENU,
    VOLUME_UP,
    VOLUME_DOWN,
    VOLUME_MUTE,
    POWER,
    MEDIA_PLAY,
    MEDIA_PAUSE,
    MEDIA_STOP,
    MEDIA_REWIND,
    MEDIA_FAST_FORWARD,
    MEDIA_PREVIOUS,
    MEDIA_NEXT,
    CHANNEL_UP,
    CHANNEL_DOWN,
    INFO,
    TV,
    GUIDE,
}

impl AndroidKeyCode {
    pub fn name(&self) -> &'static str {
        match self {
            AndroidKeyCode::DPAD_UP => "DPAD_UP",
            AndroidKeyCode::DPAD_DOWN => "DPAD_DOWN",
            AndroidKeyCode::DPAD_LEFT => "DPAD_LEFT",
            AndroidKeyCode::DPAD_RIGHT => "DPAD_RIGHT",
            AndroidKeyCode::DPAD_CENTER => "DPAD_CENTER",
            AndroidKeyCode::BACK => "BACK",
            AndroidKeyCode::HOME => "HOME",
            AndroidKeyCode::MENU => "MENU",
            AndroidKeyCode::VOLUME_UP => "VOLUME_UP",
            AndroidKeyCode::VOLUME_DOWN => "VOLUME_DOWN",
            AndroidKeyCode::VOLUME_MUTE => "VOLUME_MUTE",
            AndroidKeyCode::POWER => "POWER",
            AndroidKeyCode::MEDIA_PLAY => "MEDIA_PLAY",
            AndroidKeyCode::MEDIA_PAUSE => "MEDIA_PAUSE",
            AndroidKeyCode::MEDIA_STOP => "MEDIA_STOP",
            AndroidKeyCode::MEDIA_REWIND => "MEDIA_REWIND",
            AndroidKeyCode::MEDIA_FAST_FORWARD => "MEDIA_FAST_FORWARD",
            AndroidKeyCode::MEDIA_PREVIOUS => "MEDIA_PREVIOUS",
            AndroidKeyCode::MEDIA_NEXT => "MEDIA_NEXT",
            AndroidKeyCode::CHANNEL_UP => "CHANNEL_UP",
            AndroidKeyCode::CHANNEL_DOWN => "CHANNEL_DOWN",
            AndroidKeyCode::INFO => "INFO",
            AndroidKeyCode::TV => "TV",
            AndroidKeyCode::GUIDE => "GUIDE",
        }
    }
}

/// Source of a remote configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteSource {
    CoreElec,
    LibreElec,
    SlimBoxTv,
    AidansRom,
    Community,
    Custom,
}

/// A complete remote control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub name: String,
    pub factory_code: u32,
    pub protocol: String,
    pub repeat_period: u32,
    pub release_delay: u32,
    pub keymaps: HashMap<u8, LinuxKeyCode>,
    pub source: RemoteSource,
    pub compatible_devices: Vec<String>,
}

impl RemoteConfig {
    /// Generate Amlogic remote.conf content
    pub fn generate_remote_conf(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Remote configuration for {}\n", self.name));
        output.push_str(&format!("# Source: {:?}\n", self.source));
        output.push_str("\n[factory_code]\n");
        output.push_str(&format!("factory_code = 0x{:04X}\n", self.factory_code));
        output.push_str("\n[work_mode]\n");
        output.push_str("work_mode = 0\n");
        output.push_str("\n[protocol]\n");
        output.push_str(&format!("protocol = {}\n", self.protocol));
        output.push_str("\n[repeat]\n");
        output.push_str(&format!("repeat = {}\n", self.repeat_period));
        output.push_str("\n[release_delay]\n");
        output.push_str(&format!("release_delay = {}\n", self.release_delay));
        output.push_str("\n[key_begin]\n");

        for (scancode, keycode) in &self.keymaps {
            output.push_str(&format!("0x{:02X} {}\n", scancode, keycode.name()));
        }

        output.push_str("[key_end]\n");

        output
    }

    /// Generate Android Generic.kl keylayout content
    pub fn generate_keylayout(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Android key layout for {}\n", self.name));
        output.push_str("# This file should be placed in /system/usr/keylayout/\n\n");

        for keycode in self.keymaps.values() {
            let android_key = linux_to_android_key(keycode);
            output.push_str(&format!("key {} {}\n", keycode.as_u16(), android_key));
        }

        output
    }
}

/// Map Linux key codes to Android key names
fn linux_to_android_key(linux: &LinuxKeyCode) -> &'static str {
    match linux {
        LinuxKeyCode::KEY_UP => "DPAD_UP",
        LinuxKeyCode::KEY_DOWN => "DPAD_DOWN",
        LinuxKeyCode::KEY_LEFT => "DPAD_LEFT",
        LinuxKeyCode::KEY_RIGHT => "DPAD_RIGHT",
        LinuxKeyCode::KEY_OK | LinuxKeyCode::KEY_ENTER => "DPAD_CENTER",
        LinuxKeyCode::KEY_BACK => "BACK",
        LinuxKeyCode::KEY_HOME => "HOME",
        LinuxKeyCode::KEY_MENU => "MENU",
        LinuxKeyCode::KEY_VOLUMEUP => "VOLUME_UP",
        LinuxKeyCode::KEY_VOLUMEDOWN => "VOLUME_DOWN",
        LinuxKeyCode::KEY_MUTE => "VOLUME_MUTE",
        LinuxKeyCode::KEY_POWER => "POWER",
        LinuxKeyCode::KEY_PLAY => "MEDIA_PLAY",
        LinuxKeyCode::KEY_PAUSE => "MEDIA_PAUSE",
        LinuxKeyCode::KEY_STOP => "MEDIA_STOP",
        LinuxKeyCode::KEY_REWIND => "MEDIA_REWIND",
        LinuxKeyCode::KEY_FASTFORWARD => "MEDIA_FAST_FORWARD",
        LinuxKeyCode::KEY_PREVIOUS => "MEDIA_PREVIOUS",
        LinuxKeyCode::KEY_NEXT => "MEDIA_NEXT",
        LinuxKeyCode::KEY_CHANNELUP => "CHANNEL_UP",
        LinuxKeyCode::KEY_CHANNELDOWN => "CHANNEL_DOWN",
        LinuxKeyCode::KEY_INFO => "INFO",
        LinuxKeyCode::KEY_TV => "TV",
        LinuxKeyCode::KEY_EPG => "GUIDE",
        _ => "UNKNOWN",
    }
}

/// Database of known remote configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfigDatabase {
    pub remotes: Vec<RemoteConfig>,
}

impl RemoteConfigDatabase {
    /// Load the default database of known remotes
    pub fn default_database() -> &'static Self {
        // ⚡ Bolt: Use LazyLock to initialize the default database only once.
        // This avoids recreating a large Vec/HashMap with Strings on every call,
        // reducing access time significantly by returning a static reference.
        static DEFAULT_DB: std::sync::LazyLock<RemoteConfigDatabase> = std::sync::LazyLock::new(|| {
            let remotes = vec![
                RemoteConfig {
                    name: "X96 Mini Remote".to_string(),
                    factory_code: 0x4040,
                    protocol: "NEC".to_string(),
                    repeat_period: 125,
                    release_delay: 80,
                    keymaps: RemoteConfigDatabase::x96_mini_keymap(),
                    source: RemoteSource::Community,
                    compatible_devices: vec![
                        "X96 Mini".to_string(),
                        "X96".to_string(),
                        "TX3 Mini".to_string(),
                    ],
                },
                RemoteConfig {
                    name: "H96 Max Remote".to_string(),
                    factory_code: 0x4444,
                    protocol: "NEC".to_string(),
                    repeat_period: 125,
                    release_delay: 80,
                    keymaps: RemoteConfigDatabase::h96_max_keymap(),
                    source: RemoteSource::Community,
                    compatible_devices: vec!["H96 Max".to_string(), "H96 Pro".to_string()],
                },
                RemoteConfig {
                    name: "T95 Series Remote".to_string(),
                    factory_code: 0x00FF,
                    protocol: "NEC".to_string(),
                    repeat_period: 125,
                    release_delay: 80,
                    keymaps: RemoteConfigDatabase::t95_keymap(),
                    source: RemoteSource::Community,
                    compatible_devices: vec![
                        "T95 Max".to_string(),
                        "T95 Z Plus".to_string(),
                        "T95K Pro".to_string(),
                    ],
                },
            ];

            RemoteConfigDatabase { remotes }
        });

        &DEFAULT_DB
    }

    /// Find a remote by its factory code
    pub fn find_by_factory_code(&self, code: u32) -> Option<&RemoteConfig> {
        self.remotes.iter().find(|r| r.factory_code == code)
    }

    /// Find a remote by name (case-insensitive partial match)
    pub fn find_by_name(&self, name: &str) -> Option<&RemoteConfig> {
        let lower = name.to_lowercase();
        self.remotes
            .iter()
            .find(|r| r.name.to_lowercase().contains(&lower))
    }

    /// Get all remote names
    pub fn list_names(&self) -> Vec<&str> {
        self.remotes.iter().map(|r| r.name.as_str()).collect()
    }

    /// Load database from YAML file
    pub fn load_from_file(path: &Path) -> Result<Self, AppError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| AppError::IoError(e.to_string()))?;
        serde_yaml::from_str(&content).map_err(|e| AppError::ParseError(e.to_string()))
    }

    // Helper functions to create keymaps
    fn x96_mini_keymap() -> HashMap<u8, LinuxKeyCode> {
        let mut map = HashMap::new();
        map.insert(0x01, LinuxKeyCode::KEY_POWER);
        map.insert(0x1A, LinuxKeyCode::KEY_MUTE);
        map.insert(0x10, LinuxKeyCode::KEY_VOLUMEUP);
        map.insert(0x11, LinuxKeyCode::KEY_VOLUMEDOWN);
        map.insert(0x46, LinuxKeyCode::KEY_UP);
        map.insert(0x16, LinuxKeyCode::KEY_DOWN);
        map.insert(0x47, LinuxKeyCode::KEY_LEFT);
        map.insert(0x15, LinuxKeyCode::KEY_RIGHT);
        map.insert(0x55, LinuxKeyCode::KEY_OK);
        map.insert(0x0D, LinuxKeyCode::KEY_BACK);
        map.insert(0x06, LinuxKeyCode::KEY_HOME);
        map.insert(0x14, LinuxKeyCode::KEY_MENU);
        map.insert(0x50, LinuxKeyCode::KEY_1);
        map.insert(0x51, LinuxKeyCode::KEY_2);
        map.insert(0x52, LinuxKeyCode::KEY_3);
        map.insert(0x53, LinuxKeyCode::KEY_4);
        map.insert(0x54, LinuxKeyCode::KEY_5);
        map.insert(0x05, LinuxKeyCode::KEY_6);
        map.insert(0x02, LinuxKeyCode::KEY_7);
        map.insert(0x03, LinuxKeyCode::KEY_8);
        map.insert(0x04, LinuxKeyCode::KEY_9);
        map.insert(0x0A, LinuxKeyCode::KEY_0);
        map
    }

    fn h96_max_keymap() -> HashMap<u8, LinuxKeyCode> {
        let mut map = HashMap::new();
        map.insert(0x12, LinuxKeyCode::KEY_POWER);
        map.insert(0x10, LinuxKeyCode::KEY_MUTE);
        map.insert(0x58, LinuxKeyCode::KEY_VOLUMEUP);
        map.insert(0x59, LinuxKeyCode::KEY_VOLUMEDOWN);
        map.insert(0x40, LinuxKeyCode::KEY_UP);
        map.insert(0x41, LinuxKeyCode::KEY_DOWN);
        map.insert(0x07, LinuxKeyCode::KEY_LEFT);
        map.insert(0x06, LinuxKeyCode::KEY_RIGHT);
        map.insert(0x43, LinuxKeyCode::KEY_OK);
        map.insert(0x44, LinuxKeyCode::KEY_BACK);
        map.insert(0x45, LinuxKeyCode::KEY_HOME);
        map.insert(0x4F, LinuxKeyCode::KEY_MENU);
        map
    }

    fn t95_keymap() -> HashMap<u8, LinuxKeyCode> {
        let mut map = HashMap::new();
        map.insert(0x45, LinuxKeyCode::KEY_POWER);
        map.insert(0x00, LinuxKeyCode::KEY_UP);
        map.insert(0x01, LinuxKeyCode::KEY_DOWN);
        map.insert(0x08, LinuxKeyCode::KEY_LEFT);
        map.insert(0x5A, LinuxKeyCode::KEY_RIGHT);
        map.insert(0x1C, LinuxKeyCode::KEY_OK);
        map.insert(0x52, LinuxKeyCode::KEY_BACK);
        map.insert(0x46, LinuxKeyCode::KEY_HOME);
        map.insert(0x40, LinuxKeyCode::KEY_MENU);
        map.insert(0x44, LinuxKeyCode::KEY_VOLUMEUP);
        map.insert(0x43, LinuxKeyCode::KEY_VOLUMEDOWN);
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_remote_conf() {
        let db = RemoteConfigDatabase::default_database();
        let remote = db.find_by_name("X96").unwrap();
        let conf = remote.generate_remote_conf();

        assert!(conf.contains("factory_code = 0x4040"));
        assert!(conf.contains("KEY_POWER"));
        assert!(conf.contains("[key_begin]"));
        assert!(conf.contains("[key_end]"));
    }

    #[test]
    fn test_find_by_factory_code() {
        let db = RemoteConfigDatabase::default_database();
        let remote = db.find_by_factory_code(0x4040);
        assert!(remote.is_some());
        assert_eq!(remote.unwrap().name, "X96 Mini Remote");
    }

    #[test]
    fn test_generate_keylayout() {
        let db = RemoteConfigDatabase::default_database();
        let remote = db.find_by_name("X96").unwrap();
        let kl = remote.generate_keylayout();

        assert!(kl.contains("DPAD_CENTER") || kl.contains("POWER"));
    }
}
