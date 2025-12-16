use clap::Parser;
use git2::{Sort, Tag};
use std::process::{Command, exit};

use crate::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    #[arg(long, help = "Switch to print out the list of checkpoints as a tree")]
    tree: bool,

    #[arg(long, help = "Switch to print out the list of tags")]
    tags: bool,
}

pub fn execute(args: Args) {
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });

    if args.tree {
        // TODO: use the repo object to get the tree
        let tree_output = Command::new("git")
            .arg("tree")
            .output()
            .expect("Failed to execute command");
        let tree_stdout = String::from_utf8_lossy(&tree_output.stdout);
        println!("{}", tree_stdout);
    } else if args.tags {
        let tags = repo.tag_names(Some("*")).unwrap();

        for name_w in tags.iter() {
            let name = name_w.unwrap();
            let obj = repo.revparse_single(name).expect("Unable to get tag");

            if let Some(tag) = obj.as_tag() {
                print_tag(tag);
            } else {
                print_name(name);
            }
        }
    } else {
        // Create a revision walker
        let mut revwalk = repo.revwalk().unwrap();

        // Push the HEAD to the walker to start traversal from the current commit
        let _ = revwalk.push_head();

        // Optional: set the sorting order (default is reverse chronological by time)
        // You can use Sort::TOPOLOGICAL or combine flags like Sort::TIME | Sort::REVERSE
        revwalk
            .set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)
            .unwrap();

        // Iterate over the commit OIDs (Object IDs) returned by the walker
        for oid in revwalk {
            let oid = oid.unwrap();

            // Find the full commit object
            let commit = repo.find_commit(oid).unwrap();

            // Extract commit information
            let author = commit.author();
            let summary_bytes = commit
                .summary_bytes()
                .unwrap_or_else(|| commit.message_bytes());
            let summary = str::from_utf8(summary_bytes).unwrap_or("Invalid UTF-8 message");

            println!(
                "Commit: {}\nAuthor: {} <{}>\nSummary: {}\n",
                oid,
                author.name().unwrap_or("Unknown"),
                author.email().unwrap_or("Unknown"),
                summary,
            );
        }
    }
}

fn print_tag(tag: &Tag) {
    print!("* {:<16}", tag.name().unwrap());
    print_list_lines(tag.message());
}

fn print_name(name: &str) {
    println!("{}", name);
}

fn print_list_lines(message: Option<&str>) {
    let message = match message {
        Some(s) => s,
        _none => return,
    };
    let mut lines = message.lines().filter(|l| !l.trim().is_empty());
    if let Some(first) = lines.next() {
        print!("{}", first);
    }
    println!();
}
