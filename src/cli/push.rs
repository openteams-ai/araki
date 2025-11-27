use clap::Parser;
use git2::{Cred, PushOptions, RemoteCallbacks};
use std::process::exit;

use crate::cli::common;

#[derive(Parser, Debug, Default)]
pub struct Args {
    /// name of the tag
    #[arg()]
    tag: String,
}

pub fn execute(args: Args) {
    let repo = common::get_araki_git_repo().unwrap_or_else(|err| {
        eprintln!("Couldn't recognize the araki repo: {err}");
        exit(1);
    });
    let mut remote = repo.find_remote("origin").unwrap();

    let mut callbacks = RemoteCallbacks::new();
    // TODO: allow user to configure their ssh key
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key_from_agent(username_from_url.unwrap())
    });

    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    // Push changes
    remote
        .push(&["refs/heads/main:refs/heads/main"], Some(&mut push_opts))
        .expect("Unable to push changes");

    // Push all tags
    remote
        .push(
            &[format!("refs/tags/{}:refs/tags/{}", args.tag, args.tag)],
            Some(&mut push_opts),
        )
        .expect("Unable to push tags");
}
