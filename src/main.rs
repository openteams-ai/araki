use clap::{Parser, Subcommand};

use crate::cli::init;
use crate::cli::activate;

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
    // Initialize an environment
    Init(init::Args),
    // Activate an environment
    Activate(activate::Args),
//     // Add a package to an environment
//     Add {
//         // name of the environment, defaults to the current active environment
//         #[arg(short, long, help="Name of target environment. Defaults to the current active environment if available")]
//         name: Option<String>,
//         // names of the packages
//         #[arg(short, long, required = true, value_name = "SPEC", help="Packages to add")]
//         specs: Vec<String>,
//     },
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
        Command::Init(cmd) => init::execute(cmd),
        Command::Activate(cmd) => activate::execute(cmd),
        // Some(Command::Init { name }) => {
        //    println!("(not) initializing env: {}", name); 
        // }
        // Some(Commands::Activate { name }) => {
        //     println!("(not) activating env: {}", name); 
        // }
        // Some(Commands::Add { name, specs }) => {
        //     if let Some(n) = name {
        //         println!("(not) adding specs to '{}': {:?}", n, specs);
        //     } else {
        //         println!("(not) adding specs to current environment: {:?}", specs);
        //     }
        // }
        // Some(Commands::Save { name, tag }) => {
        //     if let Some(n) = name {
        //         println!("(not) saving env '{}': {:?}", n, tag);
        //     } else {
        //         println!("(not) adding specs to current environment: {:?}", tag);
        //     }
        // }
        // Some(Commands::List {  }) => {
        //    println!("(not) listing environments"); 
        // }
        // Some(Commands::Install { name, tag }) => {
        //     if let Some(n) = name {
        //         println!("(not) installing env '{}': {:?}", n, tag);
        //     } else {
        //         println!("(not) installing to current environment: {:?}", tag);
        //     }
        // }
        // Some(Commands::Push { tag }) => {
        //    println!("(not) pushing env: {}", tag); 
        // }
        // Some(Commands::Pull { tag }) => {
        //    println!("(not) pulling env: {}", tag); 
        // }
    }
}
