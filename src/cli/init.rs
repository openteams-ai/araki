use clap::Parser;
use console::style;
use indicatif::HumanDuration;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str::FromStr;
use std::time::Instant;

use crate::backends::{self, Backend};
use crate::common;

const ORG: &str = "nos-environments";

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct Args {
    /// Name of the lockspec
    name: String,

    /// Commit message
    #[arg(short, long, value_name = "MESSAGE")]
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
    let path_str = path.to_str().unwrap_or_else(|| {
        eprintln!("Could not convert {path:?} to a string.");
        exit(1);
    });

    if common::get_araki_git_repo().is_ok() {
        eprintln!("{path_str} is already managed by araki.");
        exit(1);
    }

    // Create a new respository
    let backend = backends::get_current_backend().unwrap_or_else(|err| {
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
                args.name, ORG,
            );
            exit(1);
        });

    // Clone the repository to the target directory. This also creates a .araki-git for tracking
    // lockspec git versions
    println!(
        "{} Cloning lockspec repository to {path_str}...",
        style("[2/4]").bold().dim(),
    );
    common::git_clone(backend.get_repo_info(ORG, &args.name).as_ssh_url(), &path).unwrap_or_else(
        |err| {
            eprintln!("Failed to clone the repository: {err}");
            exit(1);
        },
    );

    // Commit the lockspec as a new change
    println!("{} Committing lockspec...", style("[3/4]").bold().dim(),);
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
            eprintln!("Couldn't add {item} to the git index: {err}");
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
    let commit_oid = repo
        .commit(
            None,
            &author,
            &author,
            &args.message.unwrap_or("Initial commit".to_string()),
            &new_tree,
            &[],
        )
        .unwrap_or_else(|err| {
            eprintln!("Error committing changes: {err}");
            exit(1);
        });

    // Create a new (default) branch called 'main'
    let branch = repo
        .branch(
            "main",
            &repo.find_commit(commit_oid).unwrap_or_else(|err| {
                eprintln!("Unable to find the new commit: {err}");
                exit(1);
            }),
            true,
        )
        .unwrap_or_else(|err| {
            eprintln!("Unable to generate a main branch with the new commit: {err}");
            exit(1);
        });

    // Set the head to the new branch reference
    let branch_ref = branch.into_reference();
    let branch_ref_name = branch_ref.name().unwrap_or_else(|| {
        eprintln!("Could not convert branch reference into name.");
        exit(1);
    });
    repo.set_head(branch_ref_name).unwrap_or_else(|err| {
        eprintln!("Unable to set the repository head: {err}");
        exit(1);
    });

    // Push to remote
    println!(
        "{} Pushing changes to remote...",
        style("[4/4]").bold().dim(),
    );
    common::git_push("origin", "main").unwrap_or_else(|err| {
        eprintln!("Unable to push to remote: {err}");
        exit(1);
    });
    println!("Lockspec changes pushed to remote.");
    println!("Done in {}", HumanDuration(started.elapsed()));
}
