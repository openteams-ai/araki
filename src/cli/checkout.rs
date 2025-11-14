use clap::Parser;
use git2::Repository;
use std::env;
use std::process::Command;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // name of the tag
    #[arg(help = "Name of the tag")]
    tag: String,
}

pub fn execute(args: Args) {
    match env::var("PIXI_PROJECT_ROOT") {
        Ok(_val) => print!(""),
        Err(_) => println!("No project is currently activated"),
    }

    let project_env_dir = env::var("PIXI_PROJECT_ROOT").unwrap();
    // TODO: error checking to make sure the project_env_dir exists

    let repo = Repository::open(&project_env_dir).expect("Failed to open repository");

    let git_ref = if args.tag == "latest" {
        repo.find_reference("refs/heads/main")
            .expect("No tag found")
    } else {
        repo.find_reference(&format!("refs/tags/{}", args.tag))
            .expect("No tag found")
    };

    let git_ref_object = git_ref.peel(git2::ObjectType::Commit).unwrap();
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
        .current_dir(&project_env_dir)
        .output()
        .expect("Failed to execute command");
}
