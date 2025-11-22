use std::{env::current_dir, fmt::Display, fs::exists, process};

use crate::cli::common;
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
    org: Option<String>,
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
            self.org.clone().unwrap_or("openteams-ai".into()),
            self.repo
        )
    }

    /// Render the repository as an ssh URL
    fn as_ssh_url(&self) -> String {
        format!(
            "git@{}:{}/{}.git",
            self.domain.clone().unwrap_or("github.com".into()),
            self.org.clone().unwrap_or("openteams-ai".into()),
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
        r"((?<protocol>(git\+)?https?://)?(?<domain>github\.com)/)?((?<org>\w+)/)?(?<repo>\w+)",
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
        org: captures.name("org").map(|name| name.as_str().to_string()),
        repo: captures
            .name("repo")
            .ok_or("No repo name found in {env}")?
            .as_str()
            .to_string(),
    })
}

pub fn execute(args: Args) {
    let cwd = current_dir().unwrap_or_else(|err| {
        eprintln!("Could not get the current directory.\nReason: {err}");
        process::exit(1);
    });
    let target_toml = cwd.join("pixi.toml");
    let target_lock = cwd.join("pixi.lock");

    // Check that the target directory is free of pixi.lock and pixi.toml
    if exists(&target_toml).is_err() {
        eprintln!("{target_toml:?} already exists. Aborting.");
        process::exit(1);
    }
    if exists(&target_lock).is_err() {
        eprintln!("{target_lock:?} already exists. Aborting.");
        process::exit(1);
    }
    let remote = parse_repo_arg(&args.env).unwrap_or_else(|err| {
        eprintln!(
            "Could not fetch a repository from {}.\nReason: {err}",
            &args.env
        );
        process::exit(1);
    });
    let envs_dir = common::get_default_araki_envs_dir().unwrap_or_else(|| {
        eprintln!("Could not get the default araki environment directory.");
        process::exit(1);
    });

    // Clone the repository to the araki environments directory
    common::git_clone(remote.as_ssh_url(), &envs_dir).unwrap_or_else(|err| {
        eprintln!("Unable to clone the environment.\nReason: {err}");
        process::exit(1);
    });
    todo!("Hardlink the lockspec from the env dir to the specified path, or to the CWD");
}
