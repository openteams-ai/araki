use std::{
    env::current_dir,
    fmt::Display,
    path::PathBuf,
    process::{Command, exit},
    str::FromStr,
};

use crate::cli::common::{self, LockSpec};
use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug, Default)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// URL or <github org>/<repo name> of the lockspec to grab
    #[arg(value_name = "NAME")]
    env: String,

    /// Path where the lockspec should be cloned
    #[arg(short, long, value_name = "PATH")]
    path: Option<String>,
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
            self.get_protocol(),
            self.get_domain(),
            self.get_org(),
            self.get_repo(),
        )
    }

    /// Render the repository as an ssh URL
    fn as_ssh_url(&self) -> String {
        format!(
            "git@{}:{}/{}.git",
            self.get_domain(),
            self.get_org(),
            self.get_repo(),
        )
    }

    fn get_org(&self) -> String {
        self.org.clone().unwrap_or("openteams-ai".into())
    }
    fn get_repo(&self) -> String {
        self.repo.clone()
    }
    fn get_protocol(&self) -> String {
        self.protocol.clone().unwrap_or("https://".into())
    }
    fn get_domain(&self) -> String {
        self.domain.clone().unwrap_or("github.com".into())
    }
}

impl Display for RemoteRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_url())
    }
}

/// Clone the given lockspec URL
///
/// * `env`: Remote URL for an lockspec. If only <org>/<repo> is passed, the repository is
///   assumed to live on github.
fn parse_repo_arg(env: &str) -> Result<RemoteRepo, String> {
    let re = Regex::new(
        r"((?<protocol>(git\+)?https?://)?(?<domain>github\.com)/)?((?<org>[-a-zA-Z0-9_.]{1,100})/)?(?<repo>[-a-zA-Z0-9_.]{1,100}$)",
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
        eprintln!("Could not get the current directory: {err}");
        exit(1);
    });
    let path = args
        .path
        .map(|p| {
            PathBuf::from_str(&p).unwrap_or_else(|_| {
                eprintln!("{p} is not a valid path.");
                exit(1);
            })
        })
        .unwrap_or(cwd.clone());

    // Check that the target directory has no existing lockspec pixi.lock and pixi.toml
    if LockSpec::from_path(&path).is_ok() {
        eprintln!("A lockspec already exists at {path:?}. Aborting.");
        exit(1);
    }

    let remote = parse_repo_arg(&args.env).unwrap_or_else(|err| {
        eprintln!("{} is not a valid lockspec repository: {err}", &args.env);
        exit(1);
    });

    common::git_clone(remote.as_ssh_url(), &path).unwrap_or_else(|err| {
        eprintln!("Unable to clone the lockspec: {err}");
        exit(1);
    });

    if LockSpec::from_path(&path).is_err() {
        eprintln!(
            "The cloned lockspec repo is not valid. Is pixi.toml or pixi.lock missing from \
                {}/{} ?",
            remote.get_org(),
            remote.get_repo()
        );
        exit(1);
    }

    // Install the pixi project.
    // If this fails, remove the lockspec repository if it was cloned before,
    // in addition to the hardlinked files.
    let status = Command::new("pixi")
        .args(["install", "--frozen", "--locked", "--color", "always"])
        .current_dir(&path)
        .status();

    if status.is_err() || status.is_ok_and(|code| !code.success()) {
        eprintln!("Failed to install the environment with pixi.");
        match LockSpec::from_path(&path) {
            Ok(env_lockspec) => {
                env_lockspec.remove_files().unwrap_or_else(|rmerr| {
                    eprintln!("Unable to clean up the lockspec in {path:?}: {rmerr}")
                });
            }
            Err(othererr) => {
                eprintln!("Unable to clean up the lockspec in {path:?}: {othererr}")
            }
        };
        exit(1);
    }
}
