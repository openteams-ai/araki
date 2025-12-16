use clap::Parser;
use git2::Oid;
use std::process::{Command, exit};

use crate::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the tag
    #[arg(help = "Name of the tag or commit")]
    tag_or_commit: String,
}

pub fn execute(args: Args) {
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });

    let tag = args.tag_or_commit;

    let git_ref_object = if tag == "latest" {
        match repo.find_reference("refs/heads/main") {
            Ok(res) => res.peel(git2::ObjectType::Commit).unwrap(),
            Err(_err) => {
                eprintln!("Unable to find the latest commit at refs/heads/main");
                exit(1);
            }
        }
    } else {
        match repo.find_reference(&format!("refs/tags/{}", tag)) {
            Ok(res) => res.peel(git2::ObjectType::Commit).unwrap(),
            Err(_err) => {
                match repo.find_object(Oid::from_str(&tag).unwrap(), Some(git2::ObjectType::Commit))
                {
                    Ok(r) => r,
                    Err(_err) => {
                        eprintln!("Could not find tag '{}'", tag);
                        exit(1);
                    }
                }
            }
        }
    };

    let commit = git_ref_object
        .as_commit()
        .ok_or_else(|| git2::Error::from_str("Tag did not peel to a commit"))
        .unwrap();
    repo.checkout_tree(commit.as_object(), None)
        .expect("Unable to checkout tag");
    repo.set_head_detached(commit.id())
        .expect("Unable to set head");

    let _ = Command::new("pixi")
        .arg("install")
        .output()
        .expect("Failed to execute command");
}
