use clap::Parser;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use std::env::temp_dir;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use uuid::Uuid;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    /// Name of the environment
    #[arg()]
    name: String,

    /// Remote repository to pull environment from
    #[arg(long)]
    repository: Option<String>,
}

pub fn execute(args: Args) {
    println!("initializing env: {:?}", &args.name);

    // Get the araki envs dir
    let Some(araki_envs_dir) = common::get_default_araki_envs_dir() else {
        println!("error!");
        return;
    };

    // Check if the project already exists. If it does, exit
    let project_env_dir = araki_envs_dir.join(&args.name);
    if project_env_dir.exists() {
        eprintln!(
            "Environment {:?} already exists! {project_env_dir:?}",
            &args.name
        );
        return;
    }

    // Since initializing the env repository can fail in a number of different ways,
    // we clone into a temporary directory first. If that's successful, we then move it to the
    // target directory.
    let temp_path = temp_dir().join(Uuid::new_v4().to_string());
    if let Err(err) = fs::create_dir_all(&temp_path) {
        eprintln!("Unable to initialize the repote repository at {temp_path:?}. Reason: {err}",);
        exit(1);
    }
    if let Some(src) = args.repository {
        initialize_remote_git_project(src, &temp_path);
    } else {
        initialize_empty_project(&temp_path);
    }
    if fs::rename(&temp_path, &project_env_dir).is_err() {
        eprintln!("Error writing environment to {project_env_dir:?}");
        exit(1);
    }
}

pub fn initialize_remote_git_project(repo: String, project_env_dir: &Path) {
    println!("Pulling from remote repository '{}'", repo);
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
        if allowed_types.is_ssh_key() {
            Cred::ssh_key_from_agent(username).inspect(|_| {
                if tried_agent {
                    eprintln!(
                        "Unable to authenticate via ssh. Is ssh-agent running, and does it \
                        have your git credentials?"
                    );
                    exit(1);
                }
                tried_agent = true
            })
        } else {
            Err(git2::Error::from_str(
                "araki only supports ssh for git interactions. Please \
                    configure ssh-agent.",
            ))
        }
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    let _ = builder.clone(&repo, project_env_dir).map_err(|err| {
        eprintln!("Failed to clone {repo}. Reason: {err}");
        exit(1);
    });

    // TODO: validate that the project has a valid project structure.
    // That means it has a
    //  * pixi.toml or pyproject.toml with pixi config
    //  * pixi.lock

    // Install the pixi project
    let _ = Command::new("pixi")
        .arg("install")
        .current_dir(project_env_dir)
        .output()
        .expect("Failed to execute command");
}

pub fn initialize_empty_project(project_env_dir: &Path) {
    // Initialize the pixi project
    let _ = Command::new("pixi")
        .arg("init")
        .current_dir(project_env_dir)
        .status()
        .expect("Failed to execute command");

    // TODO: change this to use git2
    // Initialize the git repo
    let _ = Command::new("git")
        .arg("init")
        .arg("-b")
        .arg("main")
        .current_dir(project_env_dir)
        .status()
        .expect("Failed to execute command");

    // Install the pixi project
    let _ = Command::new("pixi")
        .arg("install")
        .current_dir(project_env_dir)
        .status()
        .expect("Failed to execute command");

    // Add initial git commit
    let _ = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(project_env_dir)
        .status()
        .expect("Failed to execute command");
    let _ = Command::new("git")
        .arg("commit")
        .args(["-m", "\"Initial commit\""])
        .current_dir(project_env_dir)
        .status()
        .expect("Failed to execute command");
}
