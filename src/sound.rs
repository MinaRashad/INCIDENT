use rodio::{Decoder, OutputStream, source::Source};
use std::fs::{self, File};
use std::io::{BufReader, Error};
use std::sync::OnceLock;
use std::time::Duration;
use rand::Rng;


struct Sound{
    source:rodio::source::Buffered<Decoder<BufReader<File>>>
}

struct Stream{
    output: OutputStream
}

static CLICKS:OnceLock<Vec<Sound>> = OnceLock::new();
static STREAM:OnceLock<OutputStream> = OnceLock::new();

pub fn init()-> Result<(), Error>{
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");

    let _ = match STREAM.set(stream_handle) {
        Ok(_)=>{},
        Err(_)=>panic!("Failed to get a handle for stream")
    };

    let asset_dir = "assets/type";
    let mut sources: Vec<Sound> = vec![];
    for entry in fs::read_dir(asset_dir)?{
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

    let _ = CLICKS.set(sources);

    Ok(())
}


pub fn click()->Option<Duration>{

    let sources = CLICKS.get()?;
    let idx = rand::rng().random_range(0..sources.len());
    let sound = &sources[idx];
    let buffered = sound.source.clone();

    let stream = STREAM.get()?;
    
    let duration = buffered.clone().total_duration();


    stream.mixer().add(buffered);

    return duration;
}