

pub const OS_LOGO_PATH:&str = "assets/images/OS_logo.png";
use toml::{self, Table, Value};
use std::{clone, fs, io::Error, path::{Path, PathBuf}};
use std::collections::HashMap;

use crate::data::{self, Password};


const DEFAULT_METADATA_FILE: &str = "default.meta";
const MAIN_METADATA_FILE:&str = "main.meta";


// metadata for files
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Entry{
    pub path:PathBuf
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata{
    pub access_level:Option<usize>,
    pub tags:Option<Vec<Tag>>,
    pub password: Option<String>,
    pub opened:bool
}

impl Metadata {
    fn new()->Metadata{
        Metadata{
            access_level:None,
            tags:None,
            password:None,
            opened:false
        }
    }
    pub fn set(&mut self, key: &str, value: String) {
        match key {
            "access_level" => {
                self.access_level = value.parse::<usize>().ok();
            }
            "tags" => {
                let tags: Vec<Tag> = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .map(|tag| Tag { name: tag })
                    .collect();

                self.tags = if tags.is_empty() { None } else { Some(tags) };
            }
            "password" => {
                self.password = if value.is_empty() { None } else { Some(value) };
            }
            "opened" => {
                if let Ok(v) = value.parse::<bool>() {
                    self.opened = v;
                }
            }
            _ => {}
        }
    }

    pub fn clear(&mut self, key: &str) {
        match key {
            "access_level" => {
                self.access_level = None;
            }
            "tags" => {
                self.tags = None;
            }
            "password" => {
                self.password = None;
            }
            "opened" => {
                self.opened = false;

            }
            _ => {}
        }
    }

    
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tag{
    name:String
}

pub fn init_metadata()->Result<(), Error>{
    // simply copy documents.meta to main.meta
    let from = Path::new(DEFAULT_METADATA_FILE);
    let to = Path::new(MAIN_METADATA_FILE);

    if !fs::exists(to)?{
        fs::copy(from, to);
    };

    Ok(())

}

pub fn metadata()->Result<HashMap<Entry, Metadata>, Error>{
    // simply copy documents.meta to main.meta
    let from = Path::new(MAIN_METADATA_FILE);    

    let data:String = fs::read(from)?
                        .iter().map(|c| *c as char)
                        .collect();
    
    let data = match  data.parse::<toml::Table>() {
        Ok(data)=>data,
        Err(e) => panic!("Failed to get metadata: {e}")
    };

    let mut map = HashMap::new();

    for file in data{
        let path = file.0;
        let value = file.1;

        let path = PathBuf::from(path);
        let entry = Entry{path:path};
        let mut metadata = Metadata::new();


        match value {
            Value::Table(map) =>{
                for data in map{

                    let key = data.0;
                    let val = data.1;

                    let val = val.to_string();
                    let key = key.to_ascii_lowercase();

                    let key = key.trim();

                    metadata.set(key, val);
                }
            }
            _ => {}
            
        }

        map.insert(entry, metadata);
        

    }
    

    Ok(map)
    

}
