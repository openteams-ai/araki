use std::{
    env::{current_dir, temp_dir},
    fmt::Display,
    fs::{exists, remove_file, rename},
    path::Path,
    process,
};
use uuid::Uuid;

use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug, Default)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// URL or <github org>/<repo name> of the environment to grab
    env: String,
}

#[derive(Debug, Default)]
pub struct RemoteRepo {
    org: String,
    repo: String,
    domain: Option<String>,
    protocol: Option<String>,
}

impl RemoteRepo {
    /// Render the repository as a git url
    fn as_url(&self) -> String {
        format!(
            "{}{}/{}/{}",
            self.protocol.clone().unwrap_or("https://".into()),
            self.domain.clone().unwrap_or("github.com".into()),
            self.org,
            self.repo
        )
    }

    /// Render the repository as an ssh URL
    fn as_ssh_url(&self) -> String {
        format!(
            "git@{}:{}/{}.git",
            self.domain.clone().unwrap_or("github.com".into()),
            self.org,
            self.repo
        )
    }
}

impl Display for RemoteRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_url())
    }
}

/// Clone the given environment URL
///
/// * `env`: Remote URL for an environment. If only <org>/<repo> is passed, the repository is
///   assumed to live on github.
fn parse_repo_arg(env: &str) -> Result<RemoteRepo, String> {
    let re = Regex::new(
        r"((?<protocol>(git\+)?https?://)?(?<domain>github\.com)/)?(?<org>\w+)/(?<repo>\w+)",
    )
    .map_err(|_| "Invalid regex for processing git url.")?;

    let captures = re
        .captures(env)
        .ok_or("Unrecognized format for repo name or URL: {env}.")?;

    Ok(RemoteRepo {
        protocol: captures
            .name("protocol")
            .map(|name| name.as_str().to_string()),
        domain: captures
            .name("domain")
            .map(|name| name.as_str().to_string()),
        org: captures
            .name("org")
            .ok_or("No org name found in {env}")?
            .as_str()
            .to_string(),
        repo: captures
            .name("repo")
            .ok_or("No repo name found in {env}")?
            .as_str()
            .to_string(),
    })
}

pub fn execute(args: Args) {
    let cwd = match current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Could not get the current directory.");
            return;
        }
    };
    let target_toml = cwd.join("pixi.toml");
    let target_lock = cwd.join("pixi.lock");

    let temp_path = temp_dir().join(Uuid::new_v4().to_string());
    let tmp_toml = temp_path.as_path().join("pixi.toml");
    let tmp_lock = temp_path.as_path().join("pixi.lock");

    // Check that the target directory is free of pixi.lock and pixi.toml
    if exists(&target_toml).is_err() {
        eprintln!("{target_toml:?} already exists. Aborting.");
        return;
    }
    if exists(&target_lock).is_err() {
        eprintln!("{target_lock:?} already exists. Aborting.");
        return;
    }

    let remote = match parse_repo_arg(&args.env) {
        Ok(result) => result,
        Err(err) => {
            eprintln!(
                "Could not fetch a repository from {}. Error: {err}",
                &args.env
            );
            return;
        }
    };

    crate::cli::init::initialize_remote_git_project(remote.as_ssh_url(), &temp_path);

    // Move from the temporary directory to the requested path
    if rename(&tmp_toml, &target_toml).is_err() {
        eprintln!("Error writing spec to {tmp_toml:?}. Aborting.");
        return;
    }
    if rename(&tmp_lock, &target_lock).is_err() {
        eprintln!("Error writing lockfile to {tmp_lock:?}. Aborting.");
        remove_lockspec(&cwd);
        return;
    }

    // Install the environment
    let mut child = match process::Command::new("pixi")
        .arg("install")
        .current_dir(&cwd)
        .spawn()
    {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Failed to start pixi. Is it installed?\nError: {err}");
            return;
        }
    };
    if child.wait().is_err() {
        eprintln!("pixi failed to install the environment. Aborting.");
        remove_lockspec(&cwd);
        return;
    }
    println!("Successfully installed {}/{}", remote.org, remote.repo);
}

/// Remove the lockspec files in the given directory.
///
/// * `dir`: Directory to remove lockspecs from.
fn remove_lockspec(dir: &Path) {
    let target_toml = dir.join("pixi.toml");
    let target_lock = dir.join("pixi.lock");

    if remove_file(&target_toml).is_err() {
        eprintln!("Cannot remove file at {target_toml:?}.")
    }
    if remove_file(&target_lock).is_err() {
        eprintln!("Cannot remove file at {target_lock:?}.")
    }
}
