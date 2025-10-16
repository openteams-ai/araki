use clap::Parser;
use std::fs;
use std::process::Command;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the environment
    #[arg(help="Name of the environment")]
    name: String,
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

    // Initialize the pixi project
    let _ = Command::new("pixi")
        .arg("init")
        .current_dir(&project_env_dir)
        .spawn()
        .expect("Failed to execute command");

    // Install the pixi project
    let _ = Command::new("pixi")
        .arg("install")
        .current_dir(&project_env_dir)
        .spawn()
        .expect("Failed to execute command");
}