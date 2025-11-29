use clap::Parser;
use indicatif::HumanDuration;
use std::time::Instant;
use console::style;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::{exit};
use std::str::FromStr;

use crate::cli::common;
use crate::backends::{self, Backend};


const ORG: &str = "openteams-ai";

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// Name of the lockspec
    name: String,

    /// Commit message
    #[arg(short, long, value_name="MESSAGE")]
    message: Option<String>,

    /// Path to the target directory
    #[arg()]
    path: Option<String>,
}

// Committing is complicated with libgit2. See
// https://users.rust-lang.org/t/how-can-i-do-git-add-some-file-rs-git-commit-m-message-git-push-with-git2-crate-on-a-bare-repo/94109/4
// for the approach used here.
pub async fn execute(args: Args) {
    let started = Instant::now();
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

    if common::get_araki_git_repo().is_ok() {
        eprintln!("{path:?} is already managed by araki.");
        exit(1);
    }

    // Create a new respository
    let backend = backends::get_current_backend()
        .unwrap_or_else(|err| {
            eprintln!("Unable to get the current backend: {err}");
            exit(1);
        });
    println!(
        "{} Creating lockspec repository at {}...",
        style("[1/4]").bold().dim(),
        backend.get_repo_info(ORG, &args.name).as_url(),
    );
    backend
        .create_repository(ORG, &args.name)
        .await
        .unwrap_or_else(|err| {
            eprintln!(
                "Error creating a new repository '{}' for organization '{}': {err}",
                args.name,
                ORG,
            );
            exit(1);
        });

    // Clone the repository to the target directory. This also creates a .araki-git for tracking
    // lockspec git versions
    println!(
        "{} Cloning lockspec repository...",
        style("[2/4]").bold().dim(),
    );
    let remote_url = backend.get_repo_info(ORG, &args.name).as_ssh_url();
    common::git_clone(remote_url.clone(), &path)
        .unwrap_or_else(|err| {
            eprintln!("Failed to clone the repository: {err}");
            exit(1);
        });

    // Commit the lockspec as a new change
    println!(
        "{} Committing lockspec...",
        style("[3/4]").bold().dim(),
    );
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });

    let mut index = repo.index().unwrap_or_else(|err| {
        eprintln!("Couln't get the index for the araki repo: {err}");
        exit(1);
    });
    for item in ["pixi.toml", "pixi.lock"] {
        index.add_path(Path::new(item)).unwrap_or_else(|err| {
            eprintln!("Couldn't add {item:?} to the git index: {err}");
            exit(1);
        });
    }
    index.write().unwrap_or_else(|err| {
        eprintln!("Couldn't write to the git index: {err}");
        exit(1);
    });

    let new_tree_oid = index.write_tree().unwrap_or_else(|err| {
        eprintln!("Failed to write the git tree from the index: {err}");
        exit(1);
    });
    let new_tree = repo.find_tree(new_tree_oid).unwrap_or_else(|err| {
        eprintln!("Unable to find the git tree associated with the new commit: {err}");
        exit(1);
    });
    let author = repo.signature().unwrap_or_else(|err| {
        eprintln!("Unable to get the author to use for the commit: {err}");
        exit(1);
    });

    let head = repo.head().unwrap_or_else(|err| {
        eprintln!("Unable to get the head commit: {err}");
        exit(1);
    });
    let parent = repo.find_commit(
        head
            .target()
            .unwrap_or_else(|| {
                eprintln!("Unable to get the OID of the HEAD commit.");
                exit(1);
            })
    ).unwrap_or_else(|err| {
        eprintln!("Unable to get the parent commit: {err}");
        exit(1);
    });

    repo
        .commit(
            Some("HEAD"),
            &author,
            &author,
            &args.message.unwrap_or("Initial commit".to_string()),
            &new_tree,
            &[&parent],
        )
        .unwrap_or_else(|err| {
            eprintln!("Error committing changes: {err}");
            exit(1);
        });

    // Push to remote
    println!(
        "{} Pushing changes to remote...",
        style("[4/4]").bold().dim(),
    );
    let mut origin = repo.remote("origin", &remote_url).unwrap_or_else(|err| {
        eprintln!("Unable to push lockspec changes to remote: {err}");
        exit(1);
    });
    origin
        .push(&["main"], None)
        .unwrap_or_else(|err| {
            eprintln!("Failed to push changes to remote: {err}");
            exit(1);
        });
    println!("Lockspec changes pushed to remote.");
    println!("Done in {}", HumanDuration(started.elapsed()));
}
