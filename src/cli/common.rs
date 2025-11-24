use directories::UserDirs;
use fs::OpenOptions;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use std::fmt::Display;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use toml::Table;

pub const ARAKI_BIN_DIR: &str = ".araki/bin";

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

/// Copy a directory recursively.
///
/// If a problem is encountered during copying, the partially-copied directory
/// will be removed.
///
/// * `from`: Path to be copied
/// * `to`: Destination of the copied directory
pub fn copy_directory(from: &PathBuf, to: &PathBuf) -> std::io::Result<()> {
    if !from.is_dir() {
        return Err(Error::new(
            ErrorKind::NotADirectory,
            format!("{from:?} is not a directory"),
        ));
    }

    if to.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("{to:?} already exists"),
        ));
    }

    fs::create_dir_all(to)?;
    for item in fs::read_dir(from)? {
        let entry = match item {
            Ok(e) => e,
            Err(ref err) => {
                fs::remove_dir_all(to)?;
                return Err(Error::other(format!(
                    "Error reading {item:?}.\nReason: {err}"
                )));
            }
        };
        if copy_fs_obj(from, &to.join(entry.file_name())).is_err() {
            // Clean up the new directory
            if to.is_dir() {
                fs::remove_dir_all(to)?;
            }
            return Err(Error::other(format!(
                "Unknown issue copying {from:?} to {to:?}."
            )));
        }
    }
    Ok(())
}

/// Copy a filesystem object from one place to another.
///
/// Directories are copied recursively.
///
/// * `from`: Path to be copied
/// * `to`: Destination of the copied object
fn copy_fs_obj(from: &PathBuf, to: &Path) -> std::io::Result<()> {
    let Some(name) = from.file_name() else {
        return Err(Error::other(format!("Can't get filename of {from:?}")));
    };

    if from.is_dir() {
        copy_directory(from, &to.join(name))
    } else {
        let _ = fs::copy(from, to.join(name));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LockSpec {
    pub path: PathBuf,
}

impl Display for LockSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lockspec: {:?}", self.path)
    }
}

impl LockSpec {
    pub fn specfile(&self) -> PathBuf {
        self.path.join("pixi.toml")
    }

    pub fn lockfile(&self) -> PathBuf {
        self.path.join("pixi.lock")
    }

    /// Construct a LockSpec from the given path.
    ///
    /// * `path`: Path to a directory containing a pixi.lock and a pixi.toml
    pub fn from_path<T>(path: T) -> Result<LockSpec, String>
    where
        T: AsRef<Path> + std::fmt::Debug,
    {
        let ls = LockSpec {
            path: path.as_ref().to_path_buf(),
        };

        if ls.files_exist() {
            Ok(ls)
        } else {
            Err(format!("No lockspec files found in {:?}", path))
        }
    }

    pub fn files_exist(&self) -> bool {
        self.lockfile().exists() && self.specfile().exists()
    }

    pub fn ensure_araki_metadata(&self, lockspec_name: &str) -> Result<(), String> {
        let specfile = self.specfile();

        let file = std::fs::read_to_string(&specfile)
            .map_err(|_| format!("Unable to read file {specfile:?}"))?;

        let mut toml_data: Table = file
            .parse()
            .map_err(|err| format!("Unable to parse {specfile:?} as valid toml.\nReason: {err}"))?;

        if toml_data.get("araki").is_none() {
            let mut araki_table = Table::new();
            araki_table.insert("lockspec_name".to_string(), lockspec_name.into());
            toml_data.insert("araki".to_string(), toml::Value::Table(araki_table));

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&specfile)
                .map_err(|err| {
                    format!(
                        "Unable to open araki config at {specfile:?} for writing.\nReason: {err}"
                    )
                })?;
            file.write_all(toml_data.to_string().as_bytes())
                .map_err(|err| {
                    format!("Unable to write araki config to {specfile:?}.\nReason: {err}")
                })?;
        }
        Ok(())
    }

    /// Remove the lockfile, specfile, and .araki-git/ directory from the given path.
    /// No error is thrown if these files don't exist.
    pub fn remove_files(&self) -> Result<(), String> {
        match fs::remove_file(self.specfile()) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        match fs::remove_file(self.lockfile()) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        match fs::remove_dir_all(self.path.join(".araki-git")) {
            Ok(_) => (),
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
        Ok(())
    }
}
