use std::{fs, path::{Path, PathBuf}, str::FromStr};

use dirs::{data_dir, data_local_dir, home_dir};
use serde::{Deserialize, Serialize};

pub fn find_ftba_dir() -> Option<PathBuf> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    let mut dir = data_local_dir()?;
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut dir = home_dir()?;

    dir.push(".ftba");
    dir.exists().then(|| dir)
}

pub fn find_prism_launcher_dir() -> Option<PathBuf> {
    let mut dir = data_dir()?;
    dir.push("PrismLauncher");
    dir.exists().then(|| dir)
}

pub fn all_minecraft_dirs() -> std::io::Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    if let Some(dir) = find_ftba_dir() {
        let instances = dir.join("instances");
        if instances.exists() {
            let edir = instances.read_dir()?;
            for e in edir {
                let e = e?;
                if !e.file_type()?.is_dir() { continue; }
                if uuid::Uuid::from_str(&e.file_name().to_string_lossy()).is_ok() {
                    dirs.push(e.path());
                }
            }
        }
    }

    if let Some(dir) = find_prism_launcher_dir() {
        let instances = dir.join("instances");
        if instances.exists() {
            let edir = instances.read_dir()?;
            for e in edir {
                let e = e?;
                if !e.file_type()?.is_dir() || e.file_name().as_encoded_bytes().starts_with(b".") { continue; }
                dirs.push(e.path().join("minecraft"));
            }
        }
    }

    Ok(dirs)
}

pub fn get_mods_dir(mcpath: &Path) -> Option<PathBuf> {
    let mods = mcpath.join("mods");
    let Ok(meta) = fs::metadata(&mods) else { return None; };
    if meta.is_dir() { Some(mods) } else { None }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ReqModDirs {
    List,
    Select(Box<Path>)
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum RespModDirs {
    Listed(Vec<PathBuf>),
    Selected
}