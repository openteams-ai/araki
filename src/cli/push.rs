use clap::Parser;
use std::process::exit;

use crate::{common, settings::Settings};

#[derive(Parser, Debug, Default)]
pub struct Args {
    /// name of the tag
    #[arg()]
    tag: String,
}

pub fn execute(args: Args, _settings: Settings) {
    common::git_push(
        "origin",
        &[
            "refs/heads/main",
            format!("refs/tags/{}", args.tag).as_str(),
        ],
    )
    .unwrap_or_else(|err| {
        eprintln!("Unable to push to remote: {err}");
        exit(1);
    })
}
