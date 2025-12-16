use clap::Parser;
use std::process::exit;

use crate::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    /// name of the tag
    #[arg()]
    tag: Option<String>,
}

pub fn execute(args: Args) {
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });

    if args.tag.is_none() {
        let mut tag_refs: Vec<String> = Vec::new();
        let tags = repo.tag_names(Some("*")).unwrap();

        for name_w in tags.iter() {
            tag_refs.push(format!("refs/tags/{}", &name_w.unwrap()));
        }
        tag_refs.push("refs/heads/main".to_string());

        let v2: Vec<&str> = tag_refs.iter().map(|s| &**s).collect();

        common::git_push("origin", v2.as_slice()).unwrap_or_else(|err| {
            eprintln!("Unable to push to remote: {err}");
            exit(1);
        })
    } else {
        common::git_push(
            "origin",
            &[
                "refs/heads/main",
                format!("refs/tags/{}", args.tag.unwrap()).as_str(),
            ],
        )
        .unwrap_or_else(|err| {
            eprintln!("Unable to push to remote: {err}");
            exit(1);
        })
    };
}
