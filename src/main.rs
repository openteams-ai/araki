use clap::{Parser, Subcommand};

use crate::cli::checkout;
use crate::cli::clone;
use crate::cli::list;
use crate::cli::pull;
use crate::cli::push;
use crate::cli::shell;
use crate::cli::shim;
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
#[command(arg_required_else_help = true)]
pub enum Command {
    /// Checkout a tag of an environment
    Checkout(checkout::Args),

    /// Clone a lockspec from a remote repository and install it in the current directory
    Clone(clone::Args),

    /// List available tags
    List(list::Args),

    /// Pull changes from the remote repo
    Pull(pull::Args),

    /// Push changes to the remote repo
    Push(push::Args),

    /// Write config to the shell
    Shell(shell::Args),

    /// Shim for pip, uv, conda, pixi. Meant to be called from shims only, to signal to araki
    /// that the user is attempting to use an unsupported env management tool
    #[command(hide = true)]
    Shim(shim::Args),

    /// Save the current version of the environment
    Tag(tag::Args),
}

pub fn main() {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Command::Checkout(cmd) => checkout::execute(cmd),
            Command::Clone(cmd) => clone::execute(cmd),
            Command::List(cmd) => list::execute(cmd),
            Command::Pull(cmd) => pull::execute(cmd),
            Command::Push(cmd) => push::execute(cmd),
            Command::Shell(cmd) => shell::execute(cmd),
            Command::Shim(cmd) => shim::execute(cmd),
            Command::Tag(cmd) => tag::execute(cmd),
        }
    } else {
        std::process::exit(2);
    }
}
