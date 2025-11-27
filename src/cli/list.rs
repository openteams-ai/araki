use clap::Parser;
use git2::Tag;
use std::process::{Command, exit};

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    #[arg(long, help = "Switch to print out the list of checkpoints as a tree")]
    tree: bool,
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
    } else {
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
    }
}

fn print_tag(tag: &Tag) {
    print!("{:<16}", tag.name().unwrap());
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
