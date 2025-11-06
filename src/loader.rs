use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct TextSource {
    pub content: String,
    pub duration: f32,
}

#[derive(Deserialize, Debug, Resource, Default)]
pub struct Config {
    pub text_size: f32,
    pub window_width: f32,
    pub camera_offset: f32,
}

#[derive(Resource)]
pub struct PresetManager {
    pub presets: HashMap<String, Vec<TextSource>>,
}

pub fn load_csv(file: &str) -> Result<Vec<TextSource>, Box<dyn Error>> {
    let mut csv_path = std::env::home_dir().unwrap();
    csv_path.push("ebb/presets/".to_string() + file);
    let file_content = std::fs::read_to_string(csv_path)?;

    let rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_content.as_bytes());

    let result: Vec<TextSource> = rdr
        .into_deserialize()
        .collect::<Result<Vec<TextSource>, csv::Error>>()?;
    Ok(result)
}

pub fn load_all_presets() -> Result<HashMap<String, Vec<TextSource>>, Box<dyn Error>> {
    let mut presets_path = std::env::home_dir().unwrap();
    presets_path.push("ebb/presets");
    
    let mut presets = HashMap::new();
    
    if !presets_path.exists() {
        println!("Warning: Presets directory not found at {:?}", presets_path);
        return Ok(presets);
    }
    
    let entries = std::fs::read_dir(&presets_path)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            if extension == "csv" {
                if let Some(file_name) = path.file_stem() {
                    if let Some(preset_name) = file_name.to_str() {
                        match load_csv(&format!("{}.csv", preset_name)) {
                            Ok(texts) => {
                                println!("Loaded preset '{}' with {} texts", preset_name, texts.len());
                                presets.insert(preset_name.to_string(), texts);
                            }
                            Err(e) => {
                                println!("Failed to load preset '{}': {}", preset_name, e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // デフォルトプリセットがない場合は追加
    if presets.is_empty() {
        println!("No presets found, adding default preset");
        presets.insert("default".to_string(), vec![
            TextSource {
                content: "Default Demo Text 1".to_string(),
                duration: 5.0,
            },
            TextSource {
                content: "Default Demo Text 2".to_string(),
                duration: 5.0,
            },
        ]);
    }
    
    Ok(presets)
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut conf_path = std::env::home_dir().unwrap();
    conf_path.push("ebb/config.toml");
    let file_content = std::fs::read_to_string(conf_path).unwrap();
    println!("{}", file_content);
    let result: Config = toml::from_str(&file_content.as_str())?;
    Ok(result)
}

pub fn unwrap_csv(f: &str) -> Vec<TextSource> {
    match load_csv(f) {
        Ok(n) => return n,
        Err(_e) => {
            println!("Err: Can't Load Preset File: {}", f);
            return vec![TextSource {
                content: "This is a Demo Text".to_string(),
                duration: 5.0,
            }];
        }
    };
}

pub fn unwrap_all_presets() -> PresetManager {
    match load_all_presets() {
        Ok(presets) => PresetManager { presets },
        Err(e) => {
            println!("Err: Can't Load Presets: {}", e);
            let mut default_presets = HashMap::new();
            default_presets.insert("default".to_string(), vec![
                TextSource {
                    content: "This is a Demo Text".to_string(),
                    duration: 5.0,
                }
            ]);
            PresetManager { presets: default_presets }
        }
    }
}

pub fn unwrap_conf() -> Config {
    match load_config() {
        Ok(n) => return n,
        Err(_e) => {
            println!("Err: Can't Load Config File!");
            return Config {
                text_size: 1080.0,
                window_width: 1920.0,
                camera_offset: 0.0,
            };
        }
    };
}
