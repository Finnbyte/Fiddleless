use std::process::Command;
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::auth::token;

fn who_am_i() -> String {
    let output = Command::new("cmd")
            .args(["/C", "whoami"])
            .output()
            .expect("failed to execute process")
            .stdout;
    let output_stringified = String::from_utf8_lossy(&output).to_string();
    let output_split: Vec<&str> = output_stringified.split("\\").collect();
    return output_split[1].trim_end().to_string();
}

pub fn build_cache_path() -> PathBuf {
    let username: String = who_am_i();
    let cache_location = format!("C:/Users/{}/AppData/Local/fiddleless/config.txt", &username);
    return Path::new(cache_location.as_str()).to_path_buf();
}

pub fn is_league_dir(potential_league_dir: PathBuf) -> bool {
    let file1 = potential_league_dir.as_path().join("LeagueClient.exe");
    let file2 = potential_league_dir.as_path().join("Game.db");
    return file1.exists() && file2.exists()
}

pub fn create_cache(data: &Path) -> Result<(), std::io::Error> {
    let cache_path = build_cache_path();
    if let Some(parent_dir) = cache_path.parent() {
        if !parent_dir.exists() {
            std::fs::create_dir(cache_path.parent().unwrap())?;
        }
    }
    let mut f = File::create(cache_path)?;
    if let Some(league_dir_location) = data.to_str() {
        f.write_fmt(format_args!("league_dir={}", league_dir_location))?;
    }
    Ok(())
}

pub fn read_cache() -> Result<PathBuf, Box<dyn std::error::Error>> {
    const KEY: &str = "league_dir";
    let data: String = std::fs::read_to_string(build_cache_path())?;
    let data_splitted: Vec<&str> = data.split("=").collect();
    
    if data_splitted.len() <= 1 || data_splitted[0] != KEY {
        return Err("Cache file has invalid format".into());
    }

    Ok(data_splitted[1].into())
}
