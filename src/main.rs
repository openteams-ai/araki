use clap::{Parser, Subcommand};

use crate::cli::activate;
use crate::cli::checkout;
use crate::cli::deactivate;
use crate::cli::envs;
use crate::cli::init;
use crate::cli::list;
use crate::cli::tag;

pub mod cli;

/// Manage and share environments
#[derive(Parser, Debug)]
#[command(author, version, about = "Manage and version pixi environments")]
pub struct Cli {
    // Manage environments
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Activate an environment
    Activate(activate::Args),

    /// Checkout a tag of an environment
    Checkout(checkout::Args),

    /// Deactivate an environment
    Deactivate(deactivate::Args),

    /// Manage environments
    Envs(envs::Args),

    /// Initialize an environment
    Init(init::Args),

    /// Save the current version of the environment
    Tag(tag::Args),

    /// List available tags
    List(list::Args),

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
        Command::Checkout(cmd) => checkout::execute(cmd),
        Command::Deactivate(cmd) => deactivate::execute(cmd),
        Command::Envs(cmd) => envs::execute(cmd),
        Command::Init(cmd) => init::execute(cmd),
        Command::List(cmd) => list::execute(cmd),
        Command::Tag(cmd) => tag::execute(cmd),
    }
}
