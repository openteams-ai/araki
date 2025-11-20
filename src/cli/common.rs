use directories::UserDirs;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use std::fs;
use std::path::{Path, PathBuf};

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

/// Clone a git repo to a path.
///
/// * `repo`: URL of a git repo to clone
/// * `path`: Path where the repo should be cloned
pub fn git_clone(repo: String, path: &Path) -> Result<(), String> {
    let mut callbacks = RemoteCallbacks::new();

    // Keep track of whether we've tried to get credentials from ssh-agent.
    // See https://github.com/nodegit/nodegit/issues/1133 for an example of this, but it affects
    // git2-rs as well; see https://github.com/rust-lang/git2-rs/issues/1140 and
    // https://github.com/rust-lang/git2-rs/issues/347 for more context.
    let mut tried_agent = false;

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        let username = username_from_url.ok_or(git2::Error::from_str(
            "Unable to get the ssh username from the URL.",
        ))?;
        if tried_agent {
            return Err(git2::Error::from_str(
                "Unable to authenticate via ssh. Is ssh-agent running, and have you \
                    added the ssh key you use for git?",
            ));
        }

        if allowed_types.is_ssh_key() {
            tried_agent = true;
            return Cred::ssh_key_from_agent(username);
        }

        Err(git2::Error::from_str(
            "araki only supports ssh for git interactions. Please configure ssh-agent.",
        ))
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    let _ = builder
        .clone(&repo, path)
        .map_err(|err| format!("Failed to clone {repo}. Reason: {err}"))?;
    Ok(())
}
