use clap::{Parser, Subcommand};

use crate::cli::activate;
use crate::cli::deactivate;
use crate::cli::envs;
use crate::cli::init;

pub mod cli;

// pub mod cli;

/// Manage and share environments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    // Manage environments
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    // Activate an environment
    Activate(activate::Args),

    // Deactivate an environment
    Deactivate(deactivate::Args),

    // Manage environments
    Envs(envs::Args),

    // Initialize an environment
    Init(init::Args),
//     // Save a checkpoint for the environment
//     Save {
//         // name of the environment, defaults to the current active environment
//         #[arg(short, long, help="Name of target environment. Defaults to the current active environment if available")]
//         name: Option<String>,
//         // name of the tag
//         #[arg(short, long, required = true, help="Name of the tag")]
//         tag: Vec<String>, 
//     },
//     // List all available environments
//     List {

//     },
//     // Install a tag into an environment
//     Install {
//         // name of the environment, defaults to the current active environment
//         #[arg(short, long, help="Name of target environment. Defaults to the current active environment if available")]
//         name: Option<String>,
//         // name of the tag to install
//         #[arg(help="Name of the tag")]
//         tag: String
//     },
//     // Push environment to a remote repo
//     Push {
//         // name of the tag to push
//         #[arg(help="Name of the tag")]
//         tag: String
//     },
//     // Pull environment from a remote repo
//     Pull {
//         // name of the tag to push
//         #[arg(help="Name of the tag")]
//         tag: String
//     },
}

pub fn main() {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        // match CI expectations
        std::process::exit(2);
    };

    match command {
        Command::Activate(cmd) => activate::execute(cmd),
        Command::Deactivate(cmd) => deactivate::execute(cmd),
        Command::Envs(cmd) => envs::execute(cmd),
        Command::Init(cmd) => init::execute(cmd),
    }
}
