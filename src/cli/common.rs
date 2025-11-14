use directories::UserDirs;
use std::fs;
use std::path::PathBuf;

pub const ARAKI_ENVS_DIR: &str = ".araki/envs";
pub const ARAKI_BIN_DIR: &str = ".araki/bin";

/// Get the user's araki envs directory, which by default
/// is placed in their home directory
pub fn get_default_araki_envs_dir() -> Option<PathBuf> {
    let Some(araki_envs_dir) = UserDirs::new().map(|dirs| dirs.home_dir().join(ARAKI_ENVS_DIR))
    else {
        return UserDirs::new().map(|dirs| dirs.home_dir().join(ARAKI_ENVS_DIR));
    };

    if !araki_envs_dir.exists() {
        println!(
            "araki envs dir does not exist. Creating it at {:?}",
            araki_envs_dir
        );
        let _ = fs::create_dir_all(araki_envs_dir);
    }

    UserDirs::new().map(|dirs| dirs.home_dir().join(ARAKI_ENVS_DIR))
}

pub fn get_default_araki_bin_dir() -> Result<PathBuf, String> {
    let dir = UserDirs::new()
        .map(|path| path.home_dir().to_path_buf().join(ARAKI_BIN_DIR))
        .ok_or("Could not determine the user home directory.")?;

    if !dir.exists() {
        println!("araki bin dir does not exist. Creating it at {dir:?}");
        fs::create_dir_all(&dir).map_err(|err| {
            eprintln!("Could not create araki bin directory at {dir:?}. Error:\n{err}");
            format!("{err}")
        })?;
    }
    Ok(dir)
}
