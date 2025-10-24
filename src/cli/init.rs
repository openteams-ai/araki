use clap::Parser;
use git2::build::RepoBuilder;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    /// Name of the environment
    #[arg()]
    name: String,

    /// Remote source to pull environment from
    #[arg(long)]
    source: Option<String>
}

pub fn execute(args: Args){
    println!("initializing env: {:?}", &args.name);
    
    // Get the akari envs dir
    let Some(akari_envs_dir) = common::get_default_akari_envs_dir()
    else {
        println!("error!");
        return
    };

    // Check if the project already exists. If it does, exit
    let project_env_dir = akari_envs_dir.join(&args.name);
    if project_env_dir.exists() {
        println!("Environment {:?} already exists!", &args.name);
        return
    }
    let _ = fs::create_dir_all(&project_env_dir);

    if let Some(src) = args.source {
        initialize_remote_git_project(src, &project_env_dir);
    } else {
        initialize_empty_project(&project_env_dir);
    }
}

pub fn initialize_remote_git_project(source: String, project_env_dir: &Path) {
    println!("Pulling from remote source '{}'", source);

    // TODO: validate source
    let mut callbacks = RemoteCallbacks::new();
    // TODO: allow user to configure their ssh key
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key_from_agent(
            username_from_url.unwrap(),
        )
    });
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_opts);

    match builder.clone(&source, project_env_dir) {
        Ok(repo) => repo,
        Err(e) => panic!(
            "Failed to clone '{}', error: {}", source, e
        ), // TODO: better error checking, surely this should not panic
    };

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
        .output()
        .expect("Failed to execute command");

    // TODO: change this to use git2
    // Initialize the git repo
    let _ = Command::new("git")
        .arg("init")
        .current_dir(project_env_dir)
        .output()
        .expect("Failed to execute command");

    // Install the pixi project
    let _ = Command::new("pixi")
        .arg("install")
        .current_dir(project_env_dir)
        .output()
        .expect("Failed to execute command");

    // Add initial git commit
    let _ = Command::new("git")
        .arg("add") 
        .arg(".")
        .current_dir(project_env_dir)
        .output()
        .expect("Failed to execute command");
    let _ = Command::new("git")
        .arg("commit")
        .arg("-m \"Initial commit\"")
        .current_dir(project_env_dir)
        .output()
        .expect("Failed to execute command");
}