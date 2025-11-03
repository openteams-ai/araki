use directories::UserDirs;
use std::path::PathBuf;
use std::fs;

const ARAKI_ENVS_DIR: &str = ".araki/envs";

/// Get the user's araki envs directory, which by default
/// is placed in their home directory
pub fn get_default_araki_envs_dir() -> Option<PathBuf> {
    let Some(araki_envs_dir) = UserDirs::new()
        .map(|dirs| dirs.home_dir().join(ARAKI_ENVS_DIR))
    else {
        return UserDirs::new()
        .map(|dirs| dirs.home_dir().join(ARAKI_ENVS_DIR))
    };

    if !araki_envs_dir.exists() {
        println!("araki envs dir does not exist. Creating it at {:?}", araki_envs_dir);
        let _ = fs::create_dir_all(araki_envs_dir);
    }

    UserDirs::new()
        .map(|dirs| dirs.home_dir().join(ARAKI_ENVS_DIR))
}
