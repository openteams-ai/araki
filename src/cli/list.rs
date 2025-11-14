use clap::Parser;
use git2::{Repository, Tag};
use std::env;
use std::process::Command;

#[derive(Parser, Debug, Default)]
pub struct Args {
    #[arg(long, help = "Switch to print out the list of checkpoints as a tree")]
    tree: bool,
}

pub fn execute(args: Args) {
    match env::var("PIXI_PROJECT_ROOT") {
        Ok(_val) => println!("Available tags:"),
        Err(_) => println!("No project is currently activated"),
    }

    let project_env_dir = env::var("PIXI_PROJECT_ROOT").unwrap();
    // TODO: error checking to make sure the project_env_dir exists

    let repo = Repository::open(&project_env_dir).expect("Failed to open repository");

    if args.tree {
        let tree_output = Command::new("git")
            .arg("tree")
            .current_dir(&project_env_dir)
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
        None => return,
    };
    let mut lines = message.lines().filter(|l| !l.trim().is_empty());
    if let Some(first) = lines.next() {
        print!("{}", first);
    }
    println!();
}
