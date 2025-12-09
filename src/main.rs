use clap::{Parser, Subcommand};
use std::process::exit;

use crate::cli::auth;
use crate::cli::checkout;
use crate::cli::clone;
use crate::cli::init;
use crate::cli::list;
use crate::cli::pull;
use crate::cli::push;
use crate::cli::shell;
use crate::cli::shim;
use crate::cli::tag;

pub mod backends;
pub mod cli;
pub mod common;
pub mod settings;

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
    /// Authenticate with the configured backend
    Auth(auth::Args),

    /// Checkout a tag of an environment
    Checkout(checkout::Args),

    /// Clone a lockspec from a remote repository and install it in the current directory
    Clone(clone::Args),

    /// Create a new araki-managed lockspec from an existing lockspec
    Init(init::Args),

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

#[tokio::main]
pub async fn main() {
    let settings = settings::get_settings_from_config_dir(
        common::get_project_dir()
            .unwrap_or_else(|err| {
                eprintln!("Couldn't get project directory: {err}");
                exit(1);
            })
            .config_dir(),
    )
    .unwrap_or_else(|err| {
        eprintln!("Couldn't get the araki settings: {err}");
        exit(1);
    });
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Command::Auth(cmd) => auth::execute(cmd, settings).await,
            Command::Checkout(cmd) => checkout::execute(cmd, settings),
            Command::Clone(cmd) => clone::execute(cmd, settings),
            Command::Init(cmd) => init::execute(cmd, settings).await,
            Command::List(cmd) => list::execute(cmd, settings),
            Command::Pull(cmd) => pull::execute(cmd, settings),
            Command::Push(cmd) => push::execute(cmd, settings),
            Command::Shell(cmd) => shell::execute(cmd, settings),
            Command::Shim(cmd) => shim::execute(cmd, settings),
            Command::Tag(cmd) => tag::execute(cmd, settings),
        }
    } else {
        std::process::exit(2);
    }
}
