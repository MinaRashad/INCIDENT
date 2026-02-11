use rodio::mixer::Mixer;
use rodio::source;
use rodio::{Decoder, OutputStream, source::Source};
use std::fs::{self, File};
use std::io::{BufReader, Error};
use std::collections::{self, HashMap, hash_map};
use std::sync::OnceLock;
use std::time::Duration;
use rand::Rng;


use crate::menu_components;


struct Sound{
    source:rodio::source::Buffered<Decoder<BufReader<File>>>
}

struct Stream{
    output: OutputStream
}

pub enum SoundCategory{
    Space,
    Type,
    Boot,
    Music,
    GUIFeedback,
}

impl SoundCategory {
    fn name(&self)->String{
        match &self {
            SoundCategory::Space => "space".to_string(),
            SoundCategory::Type => "type".to_string(),
            SoundCategory::Boot => "boot".to_string(),
            SoundCategory::Music => "music".to_string(),
            SoundCategory::GUIFeedback => "gui_feedback".to_string()
        }
    }
}


static SOUNDS:OnceLock<
        collections::HashMap<String, Vec<Sound>>
        > = OnceLock::new();
static STREAM:OnceLock<OutputStream> = OnceLock::new();

pub fn init()-> Result<(), Error>{
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");

    let _ = match STREAM.set(stream_handle) {
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

fn stream_and_sound(sound:SoundCategory)->Option<(&'static OutputStream, &'static Sound)>{
    let sound_name = sound.name();
    let map = SOUNDS.get()?;
    
    let sounds = map.get(&sound_name)?;

    let idx = rand::rng().random_range(0..sounds.len());
    let stream = STREAM.get()?;

    return Some((stream, &sounds[idx]));
}

pub fn play(sound:SoundCategory)->Option<Duration>{

    let (
        stream, 
        Sound{ source})  =
        stream_and_sound(sound)?;

    stream.mixer().add(source.clone());

    source.total_duration()
}


fn play_forever(sound:SoundCategory)->Option<()>{
    let (
        stream, 
        Sound{ source})  =
        stream_and_sound(sound)?;

    stream.mixer().add(source.clone().repeat_infinite());

    Some(())
}


pub fn keystroke_play(c:char)->Option<Duration>{

    if !c.is_whitespace() {
        play(SoundCategory::Type)
    }else{
        play(SoundCategory::Space)
    }
}

pub fn boot_play()-> Option<Duration>{
    play(SoundCategory::Boot)
}