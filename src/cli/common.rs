use directories::UserDirs;
use fs::OpenOptions;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use std::env::{current_dir, temp_dir};
use std::fmt::Display;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use toml::Table;
use uuid::Uuid;

pub const ARAKI_ENVS_DIR: &str = ".araki/envs";
pub const ARAKI_BIN_DIR: &str = ".araki/bin";
pub const ARAKI_GIT_DIR_NAME: &str = ".araki-git";

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

pub fn get_araki_git_repo() -> Result<Repository, Error> {
    let cwd = current_dir()?;
    let araki_git_dir = cwd.join(ARAKI_GIT_DIR_NAME);
    if !araki_git_dir.exists() {
        return Err(Error::other(format!(
            "No araki git dir files found in {:?}",
            araki_git_dir
        )));
    }
    Repository::open(araki_git_dir).map_err(Error::other)
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
/// The `.git/` repository gets renamed `.araki-git/`; any subsequent git commands won't target it
/// unless `--git-dir=.araki-git/` is passed as a CLI arg, or `GIT_DIR=.araki-git/` is set in the
/// environment variables.
///
/// Additionally modify `.gitignore` to ignore `.araki-git/` so that it doesn't get treated as
/// a regular file.
///
/// * `repo`: URL of a git repo to clone
/// * `path`: Path where the repo should be cloned
pub fn git_clone(repo: String, path: &Path) -> Result<(), String> {
    let temp_dir = temp_dir().join(Uuid::new_v4().to_string());
    fs::create_dir_all(&temp_dir).map_err(|err| {
        format!("Unable to clone {repo} to a temporary directory at {temp_dir:?}: {err}")
    })?;

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
        .clone(&repo, &temp_dir)
        .map_err(|err| format!("Failed to clone {repo} to {temp_dir:?}. Reason: {err}"))?;

    // Rename `.git` -> `.araki-git`
    fs::rename(temp_dir.join(".git"), temp_dir.join(ARAKI_GIT_DIR_NAME))
        .map_err(|err| format!("Error modifying the cloned repo: {err}"))?;

    copy_directory_contents(&temp_dir, &path.to_path_buf()).map_err(|err| {
        format!("Error copying the clone repo from {temp_dir:?} to {path:?}: {err}")
    })?;

    // If need be, write `.araki-git` to `.gitignore`, otherwise git treats it as a regular file
    // even if GIT_DIR is set
    let gitignore = path.join(".gitignore");
    let entry = ".araki-git/";
    if !std::fs::read_to_string(&gitignore).is_ok_and(|content| content.contains(entry)) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&gitignore)
            .map_err(|err| format!("Unable to open to {gitignore:?}: {err}"))?;

        writeln!(file, ".araki-git/")
            .map_err(|err| format!("Unable to write to {gitignore:?}: {err}"))?;
    }

    Ok(())
}

/// Copy the contents of a directory to another directory.
///
/// * `from`: Directory containing some filesystem objects
/// * `to`: Directory where they should be copied
pub fn copy_directory_contents(from: &PathBuf, to: &PathBuf) -> std::io::Result<()> {
    // Keep track of what has been copied so we can roll back if necessary
    let mut copied: Vec<PathBuf> = vec![];
    for item in fs::read_dir(from)? {
        let entry = match item {
            Ok(e) => e,
            Err(ref err) => {
                // Ignore any problems that arise during cleanup; just do our best
                let _ = remove_files(copied);
                return Err(Error::other(format!(
                    "Error reading {item:?}.\nReason: {err}"
                )));
            }
        };
        let fsobj = to.join(entry.file_name());
        if copy_fs_obj(&entry.path(), &fsobj).is_err() {
            // Ignore any problems that arise during cleanup; just do our best
            let _ = remove_files(copied);
            return Err(Error::other(format!(
                "Unknown issue copying {from:?} to {to:?}."
            )));
        }
        copied.push(fsobj);
    }
    Ok(())
}

/// Remove all the files or directories in the given vector if they exist.
///
/// Don't raise any error if a file or directory doesn't exist.
///
/// * `files`: List of files or directories to delete
pub fn remove_files(files: Vec<PathBuf>) -> std::io::Result<()> {
    for item in files {
        if item.is_dir() {
            match fs::remove_dir_all(item) {
                Ok(_) => (),
                Err(e) if e.kind() == ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            };
        } else {
            match fs::remove_file(item) {
                Ok(_) => (),
                Err(e) if e.kind() == ErrorKind::NotFound => (),
                Err(e) => return Err(e),
            };
        }
    }
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
    if from.is_dir() {
        fs::create_dir_all(to)?;
        copy_directory_contents(from, &to.to_path_buf())?;
    } else {
        let _ = fs::copy(from, to)
            .map_err(|err| Error::other(format!("Error copying {from:?} to {to:?}: {err}")))?;
    }
    Ok(())
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

    /// Check whether a lockspec (lockfile+specfile) exist in self.path.
    pub fn files_exist(&self) -> bool {
        self.lockfile().exists() && self.specfile().exists()
    }

    /// Ensure that the araki metadata containing the lockspec name is written to the specfile.
    ///
    /// * `lockspec_name`: Lockspec name to write to the file
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
