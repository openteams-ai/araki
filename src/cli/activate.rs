use clap::Parser;
use std::process::Command;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the environment
    #[arg(help="Name of the environment")]
    name: String,
}

pub fn execute(args: Args) {
    // Get the akari envs dir
    let Some(akari_envs_dir) = common::get_default_akari_envs_dir()
    else {
        println!("error!");
        return
    };

    // Check if the project already exists. If it does, exit
    let project_env_dir = akari_envs_dir.join(&args.name);
    if !project_env_dir.exists() {
        println!("Environment {:?} does not exist. Please create one using `akari init`!", &args.name);
        return
    }

    // Generate the activation script
    let activation_output = Command::new("pixi")
        .arg("shell-hook")
        .current_dir(&project_env_dir)
        .output()
        .expect("Failed to execute command");

    if !activation_output.status.success() {
        println!("Command failed with exit code: {:?}", activation_output.status.code());
        return
    }

    let activation_stdout = String::from_utf8_lossy(&activation_output.stdout);
    println!("{}", activation_stdout)
}
