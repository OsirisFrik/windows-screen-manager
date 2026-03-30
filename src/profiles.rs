use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::types::Config;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Profiles {
    #[serde(default)]
    pub profiles: BTreeMap<String, Config>,
}

fn profiles_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir()
        .ok_or("Could not determine home directory")?;
    Ok(home.join(".wsm-profiles"))
}

pub fn load_profiles() -> Result<Profiles, Box<dyn std::error::Error>> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(Profiles::default());
    }

    let contents = std::fs::read_to_string(&path)?;
    let profiles = serde_yaml::from_str(&contents)?;
    Ok(profiles)
}

pub fn save_profiles(profiles: &Profiles) -> Result<(), Box<dyn std::error::Error>> {
    let path = profiles_path()?;
    let contents = serde_yaml::to_string(&profiles)?;
    std::fs::write(&path, &contents)?;
    Ok(())
}

fn serialize_profiles(profiles: &Profiles, as_json: bool) -> Result<String, Box<dyn std::error::Error>> {
    if as_json {
        Ok(serde_json::to_string_pretty(&profiles)?)
    } else {
        Ok(serde_yaml::to_string(&profiles)?)
    }
}

pub fn get_profile(alias: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let profiles = load_profiles()?;
    profiles
        .profiles
        .get(alias)
        .cloned()
        .ok_or_else(|| format!("Profile '{}' not found", alias).into())
}

pub fn save_profile(alias: &str, config: Config, as_json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut profiles = load_profiles()?;
    profiles.profiles.insert(alias.to_string(), config);
    let path = profiles_path()?;
    let contents = serialize_profiles(&profiles, as_json)?;
    std::fs::write(&path, &contents)?;
    Ok(())
}

pub fn save_profile_to_file(
    alias: &str,
    config: Config,
    file: &std::path::Path,
    as_json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut profiles = if file.exists() {
        let contents = std::fs::read_to_string(file)?;
        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "json" => serde_json::from_str(&contents)?,
            _ => serde_yaml::from_str(&contents)?,
        }
    } else {
        Profiles::default()
    };
    profiles.profiles.insert(alias.to_string(), config);
    let contents = serialize_profiles(&profiles, as_json)?;
    std::fs::write(file, &contents)?;
    Ok(())
}

pub fn delete_profile(alias: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut profiles = load_profiles()?;
    if profiles.profiles.remove(alias).is_none() {
        return Err(format!("Profile '{}' not found", alias).into());
    }
    save_profiles(&profiles)?;
    Ok(())
}

pub fn list_profiles() -> Result<Profiles, Box<dyn std::error::Error>> {
    load_profiles()
}
