use rodio::{Decoder, OutputStream, source::Source};
use std::fs::{self, File};
use std::io::{BufReader, Error};
use std::collections::{self, HashMap};
use std::sync::OnceLock;
use std::time::Duration;
use rand::Rng;

/// Represents a buffered audio source that can be played multiple times
/// The source is pre-loaded into memory for instant playback
struct Sound{
    source:rodio::source::Buffered<Decoder<BufReader<File>>>
}

/// Categories of sounds available in the game
/// Each category corresponds to a folder in assets/sounds/
#[derive(Debug, Clone, Copy)]
pub enum SoundCategory{
    /// Space/whitespace key sounds
    Space,
    /// Typing/keystroke sounds
    Type,
    /// Boot/startup sounds
    Boot,
    /// Background music
    Music,
    /// GUI interaction feedback sounds
    GUIFeedback,
    /// Success feedback
    Good,
    /// Failed feedback
    Bad,
    /// ACCESS GRANTED (spoken)
    AccessGranted,
    /// ACCESS DENIED (spoken)
    AccessDenied
}

impl SoundCategory {
    /// Returns the folder name for this sound category
    /// Used to map categories to filesystem directories
    fn name(&self)->String{
        match &self {
            SoundCategory::Space => "space".to_string(),
            SoundCategory::Type => "type".to_string(),
            SoundCategory::Boot => "boot".to_string(),
            SoundCategory::Music => "music".to_string(),
            SoundCategory::GUIFeedback => "gui_feedback".to_string(),
            SoundCategory::Good => "good".to_string(),
            SoundCategory::Bad => "bad".to_string(),
            SoundCategory::AccessDenied => "access_denied".to_string(),
            SoundCategory::AccessGranted => "access_granted".to_string()
        }
    }
}

/// Global storage for all loaded sounds, organized by category
/// Each category can have multiple sound files for variety
static SOUNDS:OnceLock<
        collections::HashMap<String, Vec<Sound>>
        > = OnceLock::new();

/// Global audio output stream handle
/// Initialized once and reused for all audio playback
static STREAM:OnceLock<OutputStream> = OnceLock::new();

/// Initializes the audio system
/// Loads all sound files from assets/sounds/ into memory
/// Must be called before any sound playback
/// 
/// Directory structure expected:
/// assets/sounds/
///   ├── space/
///   ├── type/
///   ├── boot/
///   └── etc.
pub fn init()-> Result<(), Error>{
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");

    match STREAM.set(stream_handle) {
        Ok(_)=>{},
        Err(_)=>panic!("Failed to get a handle for stream")
    };


    let mut sound_map = HashMap::new();
    let asset_dir = "assets/sounds";
    for folder in fs::read_dir(asset_dir)?{
        let folder = folder?;
        let folderpath = folder.path();
        if !folderpath.is_dir(){
            // skip files
            continue;
        }
        let mut sources: Vec<Sound> = vec![];
        let sound_category = folderpath.file_name();
        let sound_category = match sound_category {
            Some(name)=>name,
            None=> panic!("Unable to get name of a folder")
        };
        let sound_category = match sound_category.to_str() {
            Some(name)=>name.to_owned(),
            None=> panic!("Unable to get name of a folder")

        }; 

        // add all the files buffered decoder of buffer
        // i know this sounds stupid
        for entry in fs::read_dir(folderpath)?{
            let entry = entry?;
            let path = entry.path();
            let path = path.as_path();
            let file = File::open(path)?;
            let source = Decoder::try_from(file);
            let source = match source {
                Ok(s)=> s,
                Err(_)=>panic!("Failed to decode")
            };
            let source = source.buffered();
            let sound = Sound{source};

            sources.push(sound);

        };

        sound_map.entry(sound_category).insert_entry(sources);


    }
    let _ = SOUNDS.set(sound_map);


    Ok(())
}

/// Gets a random sound from the specified category along with the output stream
/// Returns None if sounds haven't been initialized or category doesn't exist
/// Randomly selects one sound file from the category for variety
fn stream_and_sound(sound:SoundCategory)->Option<(&'static OutputStream, &'static Sound)>{
    let sound_name = sound.name();
    let map = SOUNDS.get()?;
    
    let sounds = map.get(&sound_name)?;

    let idx = rand::rng().random_range(0..sounds.len());
    let stream = STREAM.get()?;

    Some((stream, &sounds[idx]))
}

/// Plays a sound from the specified category once
/// Returns the duration of the sound if successful
/// Randomly picks one sound from the category each time
pub fn play(sound:SoundCategory)->Option<Duration>{

    let (
        stream, 
        Sound{ source})  =
        stream_and_sound(sound)?;

    stream.mixer().add(source.clone());

    source.total_duration()
}

/// Plays a sound from the specified category on infinite loop
/// Returns Some(()) if successful, None if sound system not initialized
fn play_forever(sound:SoundCategory)->Option<()>{
    let (
        stream, 
        Sound{ source})  =
        stream_and_sound(sound)?;

    stream.mixer().add(source.clone().repeat_infinite());

    Some(())
}

/// Plays appropriate keystroke sound based on character type
/// Non-whitespace characters play Type sounds
/// Whitespace characters play Space sounds
/// Returns the duration of the played sound
pub fn keystroke_play(c:char)->Option<Duration>{

    if !c.is_whitespace() {
        play(SoundCategory::Type)
    }else{
        play(SoundCategory::Space)
    }
}

/// Plays a boot/startup sound
/// Returns the duration of the played sound
pub fn boot_play()-> Option<Duration>{
    play(SoundCategory::Boot)
}