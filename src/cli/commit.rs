use clap::Parser;
use git2::Signature;
use std::path::Path;
use std::process::exit;

use crate::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    #[arg(short, long, help = "Commit message")]
    message: String,

    // name of the tag
    #[arg(short, long, help = "Tag the commit with name")]
    tag: Option<String>,

    #[arg(short, long, help = "Description of the tag")]
    describe_tag: Option<String>,
}

pub fn execute(args: Args) {
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });

    let mut index = repo.index().expect("Failed to get index");

    // Add files
    index
        .add_path(Path::new("pixi.toml"))
        .expect("unable to add pixi.toml");
    index
        .add_path(Path::new("pixi.lock"))
        .expect("unable to add pixi.lock");
    index.write().expect("Failed to write index");

    let tree_oid = index.write_tree().expect("failed to write tree");
    let tree = repo.find_tree(tree_oid).expect("failed to find tree");
    let signature = Signature::now("araki", "place@holder.com").expect("failed to get signature");
    let head = repo.head().expect("Failed to get HEAD");
    let parent_commit = repo
        .find_commit(head.target().expect("Failed to get HEAD target OID"))
        .expect("Failed to find parent commit");

    // Commit change
    repo.commit(
        Some("HEAD"),      // Update the HEAD reference
        &signature,        // Author
        &signature,        // Committer
        &args.message,     // Commit message
        &tree,             // Tree containing the staged changes
        &[&parent_commit], // Parent commit(s)
    )
    .expect("Failed to create commit");

    if let Some(ref tag) = args.tag {
        // Create tag
        // Get the OID of the commit to tag (e.g., HEAD)
        let head = repo.revparse_single("HEAD").expect("unable to find HEAD");

        let tag_message: String;
        if let Some(ref message) = args.describe_tag {
            tag_message = message.to_string();
        } else {
            tag_message = format!("araki environment tag: {}", tag)
        }

        repo.tag(
            tag,
            &head,
            &signature,
            &tag_message,
            false, // Set to false for an annotated tag, true for a lightweight tag
        )
        .expect("Unable to tag");
    }
}
