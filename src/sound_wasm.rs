//! wasm implementation of the `sound` module.
//!
//! Mirrors the public API of `src/sound.rs` but, instead of decoding audio with
//! rodio and pushing it to an OS device, it forwards play/stop requests to the
//! browser host (`wasm_host`), which uses the Web Audio API. The host loads the
//! same `assets/sounds/<category>/*` files, so behaviour matches: each `play`
//! picks a random clip from the category folder.
//!
//! Only compiled for `wasm32-wasip1` (see the `mod sound` selection in main.rs).

#![cfg(target_arch = "wasm32")]

use std::io::Error;
use std::time::Duration;

use crate::wasm_host;

/// Categories of sounds available in the game. Each maps to a folder under
/// `assets/sounds/`. Kept byte-for-byte compatible with `sound::SoundCategory`.
#[derive(Debug, Clone, Copy)]
pub enum SoundCategory {
    Space,
    Type,
    Boot,
    Music,
    GUIFeedback,
    Scroll,
    NewMessage,
    Good,
    Bad,
    AccessGranted,
    AccessDenied,
    Error,
    Sad,
    LowHumming,
    LoudHumming,
}

impl SoundCategory {
    fn name(&self) -> &'static str {
        match self {
            SoundCategory::Space => "space",
            SoundCategory::Type => "type",
            SoundCategory::Boot => "boot",
            SoundCategory::Music => "music",
            SoundCategory::GUIFeedback => "gui_feedback",
            SoundCategory::Good => "good",
            SoundCategory::Bad => "bad",
            SoundCategory::AccessDenied => "access_denied",
            SoundCategory::AccessGranted => "access_granted",
            SoundCategory::Error => "error",
            SoundCategory::Scroll => "scroll",
            SoundCategory::NewMessage => "new_message",
            SoundCategory::Sad => "sad",
            SoundCategory::LowHumming => "low_humming",
            SoundCategory::LoudHumming => "loud_humming",
        }
    }
}

/// A handle to a looping sound. Dropping it stops the loop, exactly like
/// dropping a `rodio::Sink`. `handle == 0` means "nothing playing" (used by
/// `idle_sink`).
pub struct Sink {
    handle: i32,
}

impl Sink {
    /// No-op on wasm; kept for API parity with `rodio::Sink::set_volume`.
    pub fn set_volume(&self, _volume: f32) {}
}

impl Drop for Sink {
    fn drop(&mut self) {
        if self.handle != 0 {
            wasm_host::stop_sound(self.handle);
        }
    }
}

/// Nothing to initialise — the host preloads the sound assets itself.
pub fn init() -> Result<(), Error> {
    Ok(())
}

/// Plays a random sound from the category once. The native version returns the
/// clip's duration; we don't have it here, so callers that sleep for it simply
/// fall back to their non-audio timing.
pub fn play(sound: SoundCategory) -> Option<Duration> {
    wasm_host::play_sound(sound.name(), false);
    None
}

/// Plays a random sound from the category on an infinite loop. Hold the
/// returned `Sink` to keep it playing; drop it to stop.
pub fn play_forever(sound: SoundCategory) -> Option<Sink> {
    wasm_host::play_sound(sound.name(), true).map(|handle| Sink { handle })
}

pub fn keystroke_play(c: char) -> Option<Duration> {
    if !c.is_whitespace() {
        play(SoundCategory::Type)
    } else {
        play(SoundCategory::Space)
    }
}

pub fn boot_play() -> Option<Duration> {
    play(SoundCategory::Boot)
}

/// A silent, detached sink — the wasm counterpart of `sound::idle_sink`.
pub fn idle_sink() -> Sink {
    Sink { handle: 0 }
}
